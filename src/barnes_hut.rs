//! Barnes-Hut algorithm for O(n log n) gravitational force calculation
//! 
//! Instead of computing forces between all pairs of particles O(n²),
//! we build a quadtree and approximate distant groups of particles
//! as single point masses.

use crate::physics::PhysicsObject;

/// Axis-aligned bounding box
#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: f32,      // left edge
    pub y: f32,      // bottom edge  
    pub width: f32,  // width
    pub height: f32, // height
}

impl Bounds {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    pub fn contains(&self, px: f32, py: f32) -> bool {
        px >= self.x && px < self.x + self.width &&
        py >= self.y && py < self.y + self.height
    }

    pub fn center_x(&self) -> f32 {
        self.x + self.width * 0.5
    }

    pub fn center_y(&self) -> f32 {
        self.y + self.height * 0.5
    }

    /// Get the quadrant (0=NW, 1=NE, 2=SW, 3=SE) for a point
    pub fn quadrant(&self, px: f32, py: f32) -> usize {
        let cx = self.center_x();
        let cy = self.center_y();
        let east = px >= cx;
        let north = py >= cy;
        match (north, east) {
            (true, false) => 0,  // NW
            (true, true) => 1,   // NE
            (false, false) => 2, // SW
            (false, true) => 3,  // SE
        }
    }

    /// Get bounds for a specific quadrant
    pub fn subdivide(&self, quadrant: usize) -> Bounds {
        let half_w = self.width * 0.5;
        let half_h = self.height * 0.5;
        let cx = self.center_x();
        let cy = self.center_y();
        
        match quadrant {
            0 => Bounds::new(self.x, cy, half_w, half_h),      // NW
            1 => Bounds::new(cx, cy, half_w, half_h),          // NE
            2 => Bounds::new(self.x, self.y, half_w, half_h),  // SW
            3 => Bounds::new(cx, self.y, half_w, half_h),      // SE
            _ => panic!("Invalid quadrant"),
        }
    }
}

/// A node in the Barnes-Hut quadtree
#[derive(Debug)]
pub struct QuadTree {
    pub bounds: Bounds,
    pub center_of_mass: [f32; 2],
    pub total_mass: f32,
    pub children: Option<Box<[QuadTree; 4]>>,
    /// For leaf nodes: index of the single particle, or None if empty
    pub particle_index: Option<usize>,
}

impl QuadTree {
    /// Create an empty node
    pub fn empty(bounds: Bounds) -> Self {
        Self {
            bounds,
            center_of_mass: [0.0, 0.0],
            total_mass: 0.0,
            children: None,
            particle_index: None,
        }
    }

    /// Build a quadtree from a list of particles
    pub fn build(particles: &[PhysicsObject<f32>], bounds: Bounds) -> Self {
        let mut root = Self::empty(bounds);
        
        for (i, particle) in particles.iter().enumerate() {
            root.insert(i, particle.position_vector, particle.mass);
        }
        
        root
    }

    /// Insert a particle into the tree
    pub fn insert(&mut self, index: usize, pos: [f32; 2], mass: f32) {
        // Skip particles outside bounds
        if !self.bounds.contains(pos[0], pos[1]) {
            return;
        }

        // Update center of mass
        let new_total_mass = self.total_mass + mass;
        if new_total_mass > 0.0 {
            self.center_of_mass[0] = (self.center_of_mass[0] * self.total_mass + pos[0] * mass) / new_total_mass;
            self.center_of_mass[1] = (self.center_of_mass[1] * self.total_mass + pos[1] * mass) / new_total_mass;
        }
        self.total_mass = new_total_mass;

        if self.children.is_some() {
            // Internal node: insert into appropriate child
            let quadrant = self.bounds.quadrant(pos[0], pos[1]);
            if let Some(ref mut children) = self.children {
                children[quadrant].insert(index, pos, mass);
            }
        } else if self.particle_index.is_some() {
            // Leaf node with existing particle: subdivide
            self.subdivide();
            
            // Re-insert the existing particle
            let old_idx = self.particle_index.take().unwrap();
            // We need to re-insert at the old center of mass position (approximation)
            // This is a simplification - ideally we'd store the old position
            let old_pos = self.center_of_mass;
            let old_mass = self.total_mass - mass;
            
            // Reset and rebuild
            self.total_mass = 0.0;
            self.center_of_mass = [0.0, 0.0];
            
            // We need the actual positions, so we'll use a different approach
            // Just insert the new particle into the appropriate quadrant
            let quadrant = self.bounds.quadrant(pos[0], pos[1]);
            if let Some(ref mut children) = self.children {
                // Insert both particles - but we've lost the old position info
                // This is a limitation of the current design
                // For now, insert new particle
                children[quadrant].insert(index, pos, mass);
                
                // For the old particle, use the old CoM as approximation
                let old_quadrant = self.bounds.quadrant(old_pos[0], old_pos[1]);
                children[old_quadrant].particle_index = Some(old_idx);
                children[old_quadrant].total_mass = old_mass;
                children[old_quadrant].center_of_mass = old_pos;
            }
            
            // Update this node's CoM
            self.total_mass = new_total_mass;
            self.center_of_mass[0] = (old_pos[0] * old_mass + pos[0] * mass) / new_total_mass;
            self.center_of_mass[1] = (old_pos[1] * old_mass + pos[1] * mass) / new_total_mass;
        } else {
            // Empty leaf: store particle here
            self.particle_index = Some(index);
        }
    }

    /// Subdivide this node into 4 children
    fn subdivide(&mut self) {
        self.children = Some(Box::new([
            QuadTree::empty(self.bounds.subdivide(0)),
            QuadTree::empty(self.bounds.subdivide(1)),
            QuadTree::empty(self.bounds.subdivide(2)),
            QuadTree::empty(self.bounds.subdivide(3)),
        ]));
    }

    /// Calculate gravitational force on a particle at position `pos`
    /// 
    /// - `theta`: Opening angle parameter (0.5-1.0 typical). Lower = more accurate but slower.
    /// - `g`: Gravitational constant
    /// - `softening_sq`: Softening parameter squared
    /// - `skip_index`: Index of particle to skip (self-interaction)
    pub fn calculate_force(
        &self,
        pos: [f32; 2],
        theta: f32,
        g: f32,
        softening_sq: f32,
        skip_index: usize,
    ) -> [f32; 2] {
        // Empty node contributes no force
        if self.total_mass == 0.0 {
            return [0.0, 0.0];
        }

        let dx = self.center_of_mass[0] - pos[0];
        let dy = self.center_of_mass[1] - pos[1];
        let dist_sq = dx * dx + dy * dy;
        
        // If this is a leaf with the same particle, skip
        if let Some(idx) = self.particle_index {
            if idx == skip_index {
                return [0.0, 0.0];
            }
        }

        let width = self.bounds.width.max(self.bounds.height);
        
        // Barnes-Hut criterion: if width/distance < theta, treat as point mass
        // Also treat as point mass if this is a leaf node
        let is_leaf = self.children.is_none();
        let is_far_enough = width * width < theta * theta * dist_sq;
        
        if is_leaf || is_far_enough {
            // Treat entire node as single point mass
            let dist_sq_soft = dist_sq + softening_sq;
            let dist_soft = dist_sq_soft.sqrt();
            
            // Plummer softening: a = G * m / (r² + ε²) * (dx, dy) / r
            let factor = self.total_mass * g / (dist_sq_soft * dist_soft);
            
            [dx * factor, dy * factor]
        } else {
            // Recurse into children
            let mut acc = [0.0f32, 0.0f32];
            if let Some(ref children) = self.children {
                for child in children.iter() {
                    let child_acc = child.calculate_force(pos, theta, g, softening_sq, skip_index);
                    acc[0] += child_acc[0];
                    acc[1] += child_acc[1];
                }
            }
            acc
        }
    }
}

/// Build a quadtree with proper bounds that contain all particles
pub fn build_tree(particles: &[PhysicsObject<f32>]) -> QuadTree {
    if particles.is_empty() {
        return QuadTree::empty(Bounds::new(0.0, 0.0, 1.0, 1.0));
    }

    // Find bounding box of all particles
    let mut min_x = f32::MAX;
    let mut max_x = f32::MIN;
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for p in particles {
        min_x = min_x.min(p.position_vector[0]);
        max_x = max_x.max(p.position_vector[0]);
        min_y = min_y.min(p.position_vector[1]);
        max_y = max_y.max(p.position_vector[1]);
    }

    // Add some padding and make it square
    let padding = 10.0;
    min_x -= padding;
    min_y -= padding;
    max_x += padding;
    max_y += padding;

    let width = max_x - min_x;
    let height = max_y - min_y;
    let size = width.max(height);

    let bounds = Bounds::new(min_x, min_y, size, size);
    QuadTree::build(particles, bounds)
}

