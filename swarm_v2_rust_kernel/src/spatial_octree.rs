//! TRM Swarm OS — 3D Octree Spatial Indexing Module
//! High-performance O(log N) 3D spatial partitioning for points and atoms.

#[derive(Debug, Clone, Copy)]
pub struct Point3D {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub id: u32,
}

impl Point3D {
    pub fn new(x: f32, y: f32, z: f32, id: u32) -> Self {
        Point3D { x, y, z, id }
    }

    pub fn dist_sq(&self, other: &Point3D) -> f32 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AABB {
    pub min: (f32, f32, f32),
    pub max: (f32, f32, f32),
}

impl AABB {
    pub fn new(min: (f32, f32, f32), max: (f32, f32, f32)) -> Self {
        AABB { min, max }
    }

    pub fn contains(&self, p: &Point3D) -> bool {
        p.x >= self.min.0 && p.x <= self.max.0 &&
        p.y >= self.min.1 && p.y <= self.max.1 &&
        p.z >= self.min.2 && p.z <= self.max.2
    }

    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.0 <= other.max.0 && self.max.0 >= other.min.0 &&
        self.min.1 <= other.max.1 && self.max.1 >= other.min.1 &&
        self.min.2 <= other.max.2 && self.max.2 >= other.min.2
    }
}

pub struct OctreeNode {
    pub bounds: AABB,
    pub points: Vec<Point3D>,
    pub children: Option<Box<[OctreeNode; 8]>>,
    pub capacity: usize,
}

impl OctreeNode {
    pub fn new(bounds: AABB, capacity: usize) -> Self {
        OctreeNode {
            bounds,
            points: Vec::with_capacity(capacity),
            children: None,
            capacity,
        }
    }

    pub fn insert(&mut self, point: Point3D) -> bool {
        if !self.bounds.contains(&point) {
            return false;
        }

        if self.points.len() < self.capacity && self.children.is_none() {
            self.points.push(point);
            return true;
        }

        if self.children.is_none() {
            self.subdivide();
        }

        if let Some(ref mut children) = self.children {
            for child in children.iter_mut() {
                if child.insert(point) {
                    return true;
                }
            }
        }

        false
    }

    fn subdivide(&mut self) {
        let (min_x, min_y, min_z) = self.bounds.min;
        let (max_x, max_y, max_z) = self.bounds.max;
        let mid_x = (min_x + max_x) * 0.5;
        let mid_y = (min_y + max_y) * 0.5;
        let mid_z = (min_z + max_z) * 0.5;

        let cap = self.capacity;

        let child_bounds = [
            AABB::new((min_x, min_y, min_z), (mid_x, mid_y, mid_z)),
            AABB::new((mid_x, min_y, min_z), (max_x, mid_y, mid_z)),
            AABB::new((min_x, mid_y, min_z), (mid_x, max_y, mid_z)),
            AABB::new((mid_x, mid_y, min_z), (max_x, max_y, mid_z)),
            AABB::new((min_x, min_y, mid_z), (mid_x, mid_y, max_z)),
            AABB::new((mid_x, min_y, mid_z), (max_x, mid_y, max_z)),
            AABB::new((min_x, mid_y, mid_z), (mid_x, max_y, max_z)),
            AABB::new((mid_x, mid_y, mid_z), (max_x, max_y, max_z)),
        ];

        let children = Box::new([
            OctreeNode::new(child_bounds[0], cap),
            OctreeNode::new(child_bounds[1], cap),
            OctreeNode::new(child_bounds[2], cap),
            OctreeNode::new(child_bounds[3], cap),
            OctreeNode::new(child_bounds[4], cap),
            OctreeNode::new(child_bounds[5], cap),
            OctreeNode::new(child_bounds[6], cap),
            OctreeNode::new(child_bounds[7], cap),
        ]);

        self.children = Some(children);

        // Re-insert existing points into children
        let existing = std::mem::take(&mut self.points);
        if let Some(ref mut children) = self.children {
            for pt in existing {
                for child in children.iter_mut() {
                    if child.insert(pt) {
                        break;
                    }
                }
            }
        }
    }

    pub fn query_range(&self, range: &AABB, found: &mut Vec<Point3D>) {
        if !self.bounds.intersects(range) {
            return;
        }

        for p in &self.points {
            if range.contains(p) {
                found.push(*p);
            }
        }

        if let Some(ref children) = self.children {
            for child in children.iter() {
                child.query_range(range, found);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_octree_insertion_and_query() {
        let bounds = AABB::new((-50.0, -50.0, -50.0), (50.0, 50.0, 50.0));
        let mut octree = OctreeNode::new(bounds, 4);

        for i in 0..100 {
            let p = Point3D::new(i as f32 - 50.0, 0.0, 0.0, i);
            octree.insert(p);
        }

        let query_box = AABB::new((-5.0, -5.0, -5.0), (5.0, 5.0, 5.0));
        let mut results = Vec::new();
        octree.query_range(&query_box, &mut results);

        assert!(!results.is_empty());
    }
}
