pub mod verlet_world {
    use rand::Rng;
    use std::time::{Duration, Instant};

    use crate::sim_core::verlet_object::verlet_object::VerletObject;
    use crate::sim_core::chunk::chunk::Chunk;
    use crate::sim_core::point::point::Point;

    pub struct VerletWorld {
        pub dt: f64,
        pub sub_steps: i32,
        pub objects_count: i32,
        pub step: i32,
        pub chunk_size: i32,

        pub objects: Vec<VerletObject>,
        pub chunks: Vec<Chunk>,

        pub cur_collision_resolve_duration: f64,
        pub last_collision_resolve_duration: f64,
    }

    impl VerletWorld {
        pub fn new(objects_count: i32) -> VerletWorld {
            VerletWorld {
                dt: 0.01,
                sub_steps: 8,
                objects_count: objects_count,
                chunk_size: 24,
                objects: Vec::new(),
                chunks: Vec::new(),

                step: 0,
                cur_collision_resolve_duration: 0.0,
                last_collision_resolve_duration: 0.0
            }
        }

        pub fn fill(&mut self, width_bound: f64, height_bound: f64, init_velocity_bound: f64, mass_range: std::ops::Range<f64>, radius_range: std::ops::Range<f64>) -> &mut Self {
            let mut rnd = rand::thread_rng();

            for _step in 0..self.objects_count {
                let position = (rnd.gen_range(-1.0 * width_bound .. width_bound), rnd.gen_range(-1.0 * height_bound .. height_bound));
                self.objects.push(VerletObject::new(position.0, position.1, rnd.gen_range(mass_range.clone()), rnd.gen_range(radius_range.clone()), init_velocity_bound));
            }

            return self;
        }

        pub fn update(&mut self) -> &mut Self {
            self.step += 1;
            println!("INFO: step={}, chunk_size={}", self.step, self.chunk_size);

            for _step in 0..self.sub_steps {
                let duration = self.resolve_collisions();
                self.update_objects();
                self.cur_collision_resolve_duration = (self.cur_collision_resolve_duration + duration) / 2.0;
            }

            self.update_chunk_size()
                .resolve_gravity()
                .update_objects();

            return self;
        }

        fn update_chunk_size(&mut self) -> &mut Self {
            if ((self.last_collision_resolve_duration + self.cur_collision_resolve_duration) / 2.0) < self.cur_collision_resolve_duration {
                self.chunk_size = self.chunk_size + 2;
            } else {
                self.chunk_size = self.chunk_size - 2;
            }

            if self.chunk_size < 4 {
                self.chunk_size = 4;
            }

            if self.chunk_size > 48 {
                self.chunk_size = 48;
            }

            self.last_collision_resolve_duration = self.cur_collision_resolve_duration;

            return self;
        }

        fn resolve_collisions(&mut self) -> f64 {
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
            let gravity = 6.674;
        
            for i in 0..self.objects.len() {
                for j in i..self.objects.len() {
                    if i == j {
                        continue;
                    }
        
                    let [object1, object2] = self.objects.get_many_mut([i, j]).unwrap();
        
                    let mut velocity = object1.position.minus(Point::new(object2.position.0, object2.position.1));
                    let velocity_squared = velocity.length_square();
                    let force = gravity * ((object1.mass * object2.mass) / velocity_squared);
                    let acceleration = force / f64::sqrt(velocity_squared);
        
                    object1.accelerate(
                        object2
                            .position
                            .minus(Point::new(object1.position.0, object1.position.1))
                            .multiply(acceleration)
                    );
        
                    object2.accelerate(
                        object1
                            .position
                            .minus(Point::new(object2.position.0, object2.position.1))
                            .multiply(acceleration)
                    );
                }
            }

            return self;
        }

        fn update_objects(&mut self) {
            self.chunks.clear();
        
            for object_index in 0..self.objects.len() {
                let object1 = self.objects.get_mut(object_index).unwrap();
                object1.update(self.dt / self.sub_steps as f64);

                // todo: check how to fix second borrowing error
                // self.push_to_chunks(object1.position, object_index);
            }
        }

        fn push_to_chunks(&mut self, position: Point, object_index: usize) {
            let (chunk_x, chunk_y) = position_to_chunk_coord(position, self.chunk_size);
            let chunk_position_in_vec = self.chunks.iter().position(|ch| ch.x == chunk_x && ch.y == chunk_y);

            if chunk_position_in_vec.is_none() {
                // create
                let mut indecies: Vec<i32> = Vec::new();
                indecies.push(object_index as i32);
        
                self.chunks.push(Chunk {
                    x: chunk_x,
                    y: chunk_y,
                    indecies: indecies
                });
            } else {
                // mutate
                let chunk_pos = chunk_position_in_vec.unwrap();
                let chunk = self.chunks.get_mut(chunk_pos).unwrap();

                chunk.indecies.push(object_index as i32);
            }
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
    
        object1.position = object1.position.minus(diff.multiply(object2_mass_ratio * delta).divide(2.0));
        object2.position = object2.position.plus(diff.multiply(object1_mass_ratio * delta).divide(2.0));
    
        // implementation of temperature
        // fixme: remove?
        let object1_speed = object1.position.minus(Point::new(object1.position_last.0, object1.position_last.1)).length_square();
        let object2_speed = object2.position.minus(Point::new(object2.position_last.0, object2.position_last.1)).length_square();
    
        object1.temp += common_mass * object2_speed * object2_speed * 25.0;
        object2.temp += common_mass * object1_speed * object1_speed * 25.0;
    
        let temp_to_obj1 = object2.temp * (object2_mass_ratio * 0.075);
        let temp_to_obj2 = object1.temp * (object1_mass_ratio * 0.075);
    
        object1.temp = object1.temp + temp_to_obj1 - temp_to_obj2;
        object2.temp = object2.temp + temp_to_obj2 - temp_to_obj1;

        return true;
    }

    fn position_to_chunk_coord(position: Point, chunk_size: i32) -> (i32, i32) {
        return (
            f64::floor(position.0 / f64::from(chunk_size)) as i32, 
            f64::floor(position.1 / f64::from(chunk_size)) as i32
        );
    }
}