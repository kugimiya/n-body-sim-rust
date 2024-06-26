use rand::Rng;
use std::time::{Duration, Instant};

use super::chunk::Chunk;
use super::point::Point;
use super::verlet_object::VerletObject;

pub struct VerletWorld {
    pub dt: f64,
    pub gravity_const: f64,
    pub sub_steps: i32,
    pub objects_generate_count: i32,
    pub step: i32,
    pub chunk_size: i32,
    pub costraint_radius: f64,

    pub objects: Vec<VerletObject>,
    pub chunks: Vec<Chunk>,

    pub cur_collision_resolve_duration: f64,
    pub last_collision_resolve_duration: f64,

    pub fill_allowed: bool,
    pub max_objects_count: i32,
}

impl VerletWorld {
    pub fn new(objects_count: i32, costraint_radius: f64, max_objects_count: i32) -> VerletWorld {
        VerletWorld {
            dt: 0.01,
            gravity_const: 6.674,
            sub_steps: 10,
            objects_generate_count: objects_count,
            chunk_size: 20,
            costraint_radius,
            objects: Vec::new(),
            chunks: Vec::new(),

            step: 0,
            cur_collision_resolve_duration: 0.0,
            last_collision_resolve_duration: 0.0,

            fill_allowed: true,
            max_objects_count,
        }
    }

    pub fn fill(
        &mut self,
        width_bound: f64,
        height_bound: f64,
        init_velocity_bound: f64,
        mass_range: std::ops::Range<f64>,
        radius_range: std::ops::Range<f64>,
        circled: bool
    ) -> &mut Self {
        if !self.fill_allowed {
            return self;
        }

        let mut rnd = rand::thread_rng();

        if circled {
            for _step in 1..self.objects_generate_count {
                let position = (
                    rnd.gen_range(-1.0 * width_bound .. width_bound) * f64::cos((_step as f64) / 1000.0),
                    rnd.gen_range(-1.0 * width_bound .. width_bound) * f64::sin((_step as f64) / 1000.0),
                );

                self.objects.push(VerletObject::new(
                    position.0,
                    position.1,
                    rnd.gen_range(mass_range.clone()),
                    rnd.gen_range(radius_range.clone()),
                    f64::abs(100.0 * f64::cos(_step as f64 + 0.001)),
                ));
            }
        } else {
            for _step in 0..self.objects_generate_count {
                let position = (
                    rnd.gen_range(-1.0 * width_bound..width_bound),
                    rnd.gen_range(-1.0 * height_bound..height_bound),
                );

                self.objects.push(VerletObject::new(
                    position.0,
                    position.1,
                    rnd.gen_range(mass_range.clone()),
                    rnd.gen_range(radius_range.clone()),
                    init_velocity_bound,
                ));
            }
        }

        return self;
    }

    pub fn update(&mut self) -> &mut Self {
        let time = Instant::now();
        self.step += 1;

        self.update_chunk_size();

        for _step in 0..self.sub_steps {
            let duration = self.resolve_collisions();
            self.update_objects();
            self.cur_collision_resolve_duration = (self.cur_collision_resolve_duration + duration) / 2.0;
        }

        self.resolve_gravity();
        self.apply_constraints();

        let duration: Duration = time.elapsed();
        if self.objects.len() >= self.max_objects_count as usize {
            self.fill_allowed = false;
        } else {
            self.fill_allowed = true;
        }

        self.update_objects();

        println!("INFO: step={}, chunk_size={}, chunk_count={}, object_count={}, frame_time={:?}", self.step, self.chunk_size, self.chunks.len(), self.objects.len(), duration);
        return self;
    }

    pub fn apply_constraints(&mut self) -> &mut Self {
        let mut contraint_center = Point::new(0.0, 0.0);

        for object in self.objects.iter_mut() {
            let mut velocity = contraint_center.minus(object.position);
            let distance = velocity.length();

            if distance > self.costraint_radius - object.radius {
                let mut diff = velocity.divide(distance);
                object.position_last = object.position.clone();
                object.position = contraint_center.minus(diff.multiply(self.costraint_radius - object.radius));
            }

            // so, box :^)
            if object.position.0 > self.costraint_radius * 2.0 {
                object.position.0 = self.costraint_radius;
                object.position_last.0 = self.costraint_radius;
            }
        
            if object.position.1 > self.costraint_radius * 2.0 {
                object.position.1 = self.costraint_radius;
                object.position_last.1 = self.costraint_radius;
            }
        
            if object.position.0 < self.costraint_radius * -2.0 {
                object.position.0 = self.costraint_radius * -1.0;
                object.position_last.0 = self.costraint_radius * -1.0;
            }
        
            if object.position.1 < self.costraint_radius * -2.0 {
                object.position.1 = self.costraint_radius * -1.0;
                object.position_last.1 = self.costraint_radius * -1.0;
            }
        }

        return self;
    }

    fn update_chunk_size(&mut self) -> &mut Self {
        if ((self.last_collision_resolve_duration + self.cur_collision_resolve_duration) / 2.0)
            < self.cur_collision_resolve_duration
        {
            self.chunk_size = self.chunk_size + 2;
        } else {
            self.chunk_size = self.chunk_size - 2;
        }

        if self.chunk_size < 2 {
            self.chunk_size = 2;
        }

        if self.chunk_size > 48 {
            self.chunk_size = 48;
        }

        self.last_collision_resolve_duration = self.cur_collision_resolve_duration;

        return self;
    }

    fn resolve_collisions(&mut self) -> f64 {
        let start = Instant::now();

        for chunk_index in 0..self.chunks.len() {
            let chunk = self.chunks.get(chunk_index).unwrap();
            let hashes: [(i32, i32); 5] = [
                (chunk.x, chunk.y),
                (chunk.x - 1, chunk.y),
                (chunk.x + 1, chunk.y),
                (chunk.x, chunk.y - 1),
                (chunk.x, chunk.y + 1),
            ];
    
            let mut object_indecies: Vec<i32> = Vec::new();
    
            for chunk_hash in hashes {
                let search_result = self.chunks.iter().find(|ch| ch.x == chunk_hash.0 && ch.y == chunk_hash.1);
                if !search_result.is_none() {
                    let chunk = search_result.unwrap();
                    for i in chunk.indecies.iter() {
                        if !object_indecies.contains(i) || object_indecies.len() == 0 {
                            object_indecies.push(i + 0);
                        }
                    }
                }
            }
    
            for i in 0..object_indecies.len() {
                for j in i..object_indecies.len() {
                    if i == j {
                        continue;
                    }
    
                    let get_result = self.objects.get_many_mut([object_indecies[i] as usize, object_indecies[j] as usize]);
    
                    if get_result.is_err() {
                        continue;
                    } else {
                        let [object1, object2] = get_result.unwrap();
                        apply_collisions(object1, object2);
                    }
                }
            }
        }

        let duration: Duration = start.elapsed();
        return duration.as_millis() as f64;
    }

    fn resolve_collisions_bruteforce(&mut self) -> f64 {
        let start = Instant::now();

        for i in 0..self.objects.len() {
            for j in i..self.objects.len() {
                if i == j {
                    continue;
                }

                let [object1, object2] = self.objects.get_many_mut([i, j]).unwrap();

                apply_collisions(object1, object2);
            }
        }

        let duration: Duration = start.elapsed();
        return duration.as_millis() as f64;
    }

    fn resolve_gravity(&mut self) -> &mut Self {
        for chunk_index_i in 0 .. self.chunks.len() {
            for chunk_index_j in chunk_index_i .. self.chunks.len() {
                if chunk_index_i == chunk_index_j {
                    continue;
                }

                let [chunk1, chunk2] = self.chunks.get_many_mut([chunk_index_i, chunk_index_j]).unwrap();
                for object1_index in chunk1.indecies.iter() {
                    let object1 = self.objects.get_mut(*object1_index as usize).unwrap();

                    let mut velocity = object1.position.minus(chunk2.mass_center);
                    let velocity_squared = velocity.length_square();
                    let force = self.gravity_const * ((object1.mass * chunk2.mass) / velocity_squared);
                    let acceleration = force / f64::sqrt(velocity_squared);
                    let object_acc = Point::new(chunk2.mass_center.0, chunk2.mass_center.1).minus(Point::new(object1.position.0, object1.position.1)).multiply(acceleration);
                    object1.accelerate(object_acc);
                }
            }

            let chunk = self.chunks.get(chunk_index_i).unwrap();
            for i in 0 .. chunk.indecies.len() {
                for j in i .. chunk.indecies.len() {
                    if i == j {
                        continue;
                    }

                    let [object1, object2] = self.objects.get_many_mut([i, j]).unwrap();

                    let mut velocity = object1.position.minus(Point::new(object2.position.0, object2.position.1));
                    let velocity_squared = velocity.length_square();
                    let force = self.gravity_const * ((object1.mass * object2.mass) / velocity_squared);
                    let acceleration = force / f64::sqrt(velocity_squared);
    
                    let object1_acc = object2.position.minus(Point::new(object1.position.0, object1.position.1)).multiply(acceleration);
                    let object2_acc = object1.position.minus(Point::new(object2.position.0, object2.position.1)).multiply(acceleration);
    
                    object1.accelerate(object1_acc);
                    object2.accelerate(object2_acc);
                }
            }
        }

        return self;
    }

    fn resolve_gravity_bruteforce(&mut self) -> &mut Self {
        for i in 0 .. self.objects.len() as usize {
            for j in i .. self.objects.len() as usize {
                if i == j {
                    continue;
                }

                let [object1, object2] = self.objects.get_many_mut([i, j]).unwrap();

                let mut velocity = object1.position.minus(Point::new(object2.position.0, object2.position.1));
                let velocity_squared = velocity.length_square();
                let force = self.gravity_const * ((object1.mass * object2.mass) / velocity_squared);
                let acceleration = force / f64::sqrt(velocity_squared);

                let object1_acc = object2.position.minus(Point::new(object1.position.0, object1.position.1)).multiply(acceleration);
                let object2_acc = object1.position.minus(Point::new(object2.position.0, object2.position.1)).multiply(acceleration);

                object1.accelerate(object1_acc);
                object2.accelerate(object2_acc);
            }
        }

        return self;
    }

    fn update_objects(&mut self) {
        self.chunks.clear();

        for object in self.objects.iter_mut() {
            // hot fix irrational acceleration
            if !object.acceleration.0.is_normal() {
                object.acceleration.0 = 0.0;
            }

            if !object.acceleration.1.is_normal() {
                object.acceleration.1 = 0.0;
            }

            object.update(self.dt / self.sub_steps as f64);
            object.update_friction();
            object.temp_fix();
        }

        for object_index in 0..self.objects.len() {
            self.push_to_chunks(object_index);
        }
    }

    fn push_to_chunks(&mut self, object_index: usize) -> &mut Self {
        let object: &mut VerletObject = self.objects.get_mut(object_index).unwrap();
        let (chunk_x, chunk_y) = position_to_chunk_coord(object, self.chunk_size);
        let chunk_position_in_vec = self.chunks.iter().position(|ch| ch.x == chunk_x && ch.y == chunk_y);
        if chunk_position_in_vec.is_none() {
            // create
            let mut indecies: Vec<i32> = Vec::new();
            indecies.push(object_index as i32);

            self.chunks.push(Chunk {
                x: chunk_x,
                y: chunk_y,
                indecies: indecies,
                mass_center: object.position.clone(),
                mass: object.mass
            });
        } else {
            // andrew mutate :^)
            let chunk_pos = chunk_position_in_vec.unwrap();
            let chunk = self.chunks.get_mut(chunk_pos).unwrap();
            chunk.indecies.push(object_index as i32);
            chunk.mass += object.mass;
            chunk.mass_center.0 = (chunk.mass_center.0 + object.position.0) / 2.0;
            chunk.mass_center.1 = (chunk.mass_center.1 + object.position.1) / 2.0;
        }

        return self;
    }
}

fn apply_collisions(object1: &mut VerletObject, object2: &mut VerletObject) -> bool {
    let collide_responsibility = 0.375;
    let mut velocity = object1
        .position
        .minus(Point::new(object2.position.0, object2.position.1));
    let distance_squared = velocity.length_square();
    let distance_minimal = object1.radius + object2.radius;

    // no overlap, skip
    if distance_squared >= (distance_minimal * distance_minimal) {
        return false;
    }

    let distance = f64::sqrt(distance_squared);
    let mut diff = velocity.divide(distance);

    let common_mass = object1.mass + object2.mass;
    let object1_mass_ratio = object1.mass / common_mass;
    let object2_mass_ratio = object2.mass / common_mass;

    let delta = collide_responsibility * (distance - distance_minimal);

    object1.position = object1
        .position
        .minus(diff.multiply(object2_mass_ratio * delta).divide(2.0));
    object2.position = object2
        .position
        .plus(diff.multiply(object1_mass_ratio * delta).divide(2.0));

    // hot fix for irrational value
    if !object1.position.0.is_normal() || !object1.position.1.is_normal() {
        object1.position = Point::new(0.0, 0.0);
        object1.position_last = object1.position.clone();
    }

    if !object2.position.0.is_normal() || !object2.position.1.is_normal() {
        object2.position = Point::new(0.0, 0.0);
        object2.position_last = object2.position.clone();
    }

    // implementation of temperature
    let object1_speed = object1
        .position
        .minus(Point::new(object1.position_last.0, object1.position_last.1))
        .length_square();
    let object2_speed = object2
        .position
        .minus(Point::new(object2.position_last.0, object2.position_last.1))
        .length_square();

    object1.temp += common_mass * object2_speed * object2_speed * 25.0;
    object2.temp += common_mass * object1_speed * object1_speed * 25.0;

    let temp_to_obj1 = object2.temp * (object2_mass_ratio * 0.075);
    let temp_to_obj2 = object1.temp * (object1_mass_ratio * 0.075);

    object1.temp = object1.temp + temp_to_obj1 - temp_to_obj2;
    object2.temp = object2.temp + temp_to_obj2 - temp_to_obj1;

    return true;
}

fn position_to_chunk_coord(object: &mut VerletObject, chunk_size: i32) -> (i32, i32) {
    return (
        f64::floor(object.position.0 / f64::from(chunk_size)) as i32, 
        f64::floor(object.position.1 / f64::from(chunk_size)) as i32
    );
}
