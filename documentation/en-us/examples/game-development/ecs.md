# Entity Component System (ECS)

Valkyrie provides a high-performance Entity Component System (ECS) implementation, which is the core architectural pattern of modern game engines. ECS decomposes game objects into Entities, Components, and Systems, achieving high modularity and performance optimization.

## Core Concepts

### Entity
An entity is a unique identifier for objects in the game world, containing no data or behavior itself.

### Component
Components are pure data structures describing entity attributes and states.

### System
Systems contain game logic, operating on entities with specific component combinations.

## Basic ECS Implementation

```valkyrie
use valkyrie::ecs::*

# Define components
class Position {
    x: f64,
    y: f64,
    z: f64,
}

class Velocity {
    dx: f64,
    dy: f64,
    dz: f64,
}

class Health {
    current: i32,
    maximum: i32,
}

class Sprite {
    texture_id: utf8,
    width: f32,
    height: f32,
    color: Color,
}

class Transform {
    position: Vec3,
    rotation: Quaternion,
    scale: Vec3,
}

# Create ECS world
let world = World::new()

# Create entity and add components
let player = world.spawn()
    .with(Position { x: 0.0, y: 0.0, z: 0.0, })
    .with(Velocity { dx: 0.0, dy: 0.0, dz: 0.0, })
    .with(Health { current: 100, maximum: 100, })
    .with(Sprite { texture_id: "player.png", width: 32.0, height: 32.0, color: Color::WHITE, })
    .id()

let enemy = world.spawn()
    .with(Position { x: 100.0, y: 50.0, z: 0.0, })
    .with(Velocity { dx: -10.0, dy: 0.0, dz: 0.0, })
    .with(Health { current: 50, maximum: 50, })
    .with(Sprite {
        texture_id: "enemy.png",
        width: 24.0,
        height: 24.0,
        color: Color::RED,
    })
    .id()
```

## System Implementation

```valkyrie
# Movement system
class MovementSystem;

imply MovementSystem: System {
    micro run(self, mut world: World, delta_time: f64) {
        # Query all entities with Position and Velocity components
        for (entity, (pos, vel)) in world.query⟨(mut Position, Velocity)⟩() {
            pos.x += vel.dx * delta_time
            pos.y += vel.dy * delta_time
            pos.z += vel.dz * delta_time
        }
    }
}

# Render system
class RenderSystem {
    renderer: Renderer,
}

imply RenderSystem: System {
    micro run(self, world: World, _delta_time: f64) {
        # Query all renderable entities
        for (entity, (pos, sprite)) in world.query⟨(Position, Sprite)⟩() {
            self.renderer.draw_sprite(
                sprite.texture_id,
                Vec2 { x: pos.x, y: pos.y, },
                Vec2 { x: sprite.width, y: sprite.height, },
                sprite.color,
            )
        }
    }
}

# Collision detection system
class CollisionSystem;

imply CollisionSystem: System {
    micro run(self, mut world: World, _delta_time: f64) {
        let entities: [(Entity, Position, Sprite)] = 
            world.query⟨(Position, Sprite)⟩().collect()
        
        for i in 0..entities.length {
            for j in i + 1..entities.length {
                let (e1, p1, s1) = entities[i]
                let (e2, p2, s2) = entities[j]
                
                if self.check_collision(p1, s1, p2, s2) {
                    world.send_event(CollisionEvent {
                        entity1: e1,
                        entity2: e2,
                    })
                }
            }
        }
    }
    
    micro check_collision(self, pos1: Position, sprite1: Sprite,
                          pos2: Position, sprite2: Sprite) -> bool {
        let dx = abs(pos1.x - pos2.x)
        let dy = abs(pos1.y - pos2.y)
        
        return dx < (sprite1.width + sprite2.width) / 2.0 &&
               dy < (sprite1.height + sprite2.height) / 2.0
    }
}
```

## Advanced ECS Features

### Component Tags and Markers

```valkyrie
# Marker components (no data)
class Player;  # Mark entity as player
class Enemy;   # Mark entity as enemy
class Bullet;  # Mark entity as bullet
class Collectible;  # Mark entity as collectible

# Use marker components for queries
class PlayerControlSystem;

imply PlayerControlSystem: System {
    micro run(self, mut world: World, input: Input) {
        # Only process player entities
        for (entity, (pos, vel)) in world.query⟨(mut Position, mut Velocity)⟩()
                                              .with⟨Player⟩() {
            if input.is_key_pressed(Key::W) {
                vel.dy = 100.0
            }
            if input.is_key_pressed(Key::S) {
                vel.dy = -100.0
            }
            if input.is_key_pressed(Key::A) {
                vel.dx = -100.0
            }
            if input.is_key_pressed(Key::D) {
                vel.dx = 100.0
            }
        }
    }
}
```

### Resource System

```valkyrie
# Global resources
class GameTime {
    total_time: f64,
    delta_time: f64,
    frame_count: i32,
}

class Score {
    value: i32,
    high_score: i32,
}

class AssetManager {
    textures: HashMap⟨utf8, Texture⟩,
    sounds: HashMap⟨utf8, Sound⟩,
    fonts: HashMap⟨utf8, Font⟩,
}

# Use resources in systems
class ScoreSystem;

imply ScoreSystem: System {
    micro run(self, mut world: World, _delta_time: f64) {
        let mut score = world.get_resource_mut⟨Score⟩()
        let game_time = world.get_resource⟨GameTime⟩()
        
        # Increase score every second
        if game_time.frame_count % 60 == 0 {
            score.value += 10
            if score.value > score.high_score {
                score.high_score = score.value
            }
        }
    }
}
```

### Event System

```valkyrie
# Define events
class CollisionEvent {
    entity1: Entity,
    entity2: Entity,
}

class PlayerDeathEvent {
    player: Entity,
    cause: utf8,
}

class ScoreEvent {
    points: i32,
    source: Entity,
}

# Event handling system
class EventHandlerSystem;

imply EventHandlerSystem: System {
    micro run(self, mut world: World, _delta_time: f64) {
        # Handle collision events
        for event in world.read_events⟨CollisionEvent⟩() {
            let e1_has_player = world.has_component⟨Player⟩(event.entity1)
            let e2_has_enemy = world.has_component⟨Enemy⟩(event.entity2)
            
            if e1_has_player && e2_has_enemy {
                # Player collided with enemy
                if let Some(mut health) = world.get_component_mut⟨Health⟩(event.entity1) {
                    health.current -= 10
                    if health.current <= 0 {
                        world.send_event(PlayerDeathEvent {
                            player: event.entity1,
                            cause: "enemy_collision",
                        })
                    }
                }
            }
        }
        
        # Handle player death events
        for event in world.read_events⟨PlayerDeathEvent⟩() {
            print("Player died: {}", event.cause)
            world.despawn(event.player)
        }
    }
}
```

## Performance Optimization

### Component Storage Optimization

```valkyrie
# Use SoA (Structure of Arrays) storage
class PositionStorage {
    x_values: [f64],
    y_values: [f64],
    z_values: [f64],
    entities: [Entity],
}

# Batch processing
class BatchMovementSystem;

imply BatchMovementSystem: System {
    micro run(self, mut world: World, delta_time: f64) {
        # Get all position and velocity data
        let positions = world.get_component_storage_mut⟨Position⟩()
        let velocities = world.get_component_storage⟨Velocity⟩()
        
        # Use SIMD for batch computation
        for i in 0..positions.length {
            positions.x_values[i] += velocities.dx_values[i] * delta_time
            positions.y_values[i] += velocities.dy_values[i] * delta_time
            positions.z_values[i] += velocities.dz_values[i] * delta_time
        }
    }
}
```

### Parallel System Execution

```valkyrie
use valkyrie::threading::*

# Parallel system scheduler
class ParallelScheduler {
    thread_pool: ThreadPool,
}

imply ParallelScheduler {
    micro run_systems(self, mut world: World, systems: [Box⟨System⟩]) {
        # Analyze system dependencies
        let dependency_graph = self.analyze_dependencies(systems)
        
        # Execute independent systems in parallel
        let batches = self.create_execution_batches(dependency_graph)
        
        for batch in batches {
            self.thread_pool.scope {
                for system in batch {
                    $.spawn {
                        system.run(world, delta_time)
                    }
                }
            }
        }
    }
}
```

## Complete Game Example

```valkyrie
# Simple space shooter game
class SpaceShooterGame {
    world: World,
    systems: [Box⟨System⟩],
    input: Input,
    renderer: Renderer,
}

imply SpaceShooterGame {
    micro new() -> Self {
        let mut world = World::new()
        
        # Add resources
        world.insert_resource(GameTime { total_time: 0.0, delta_time: 0.0, frame_count: 0, })
        world.insert_resource(Score { value: 0, high_score: 0, })
        
        let player = world.spawn()
            .with(Position { x: 400.0, y: 500.0, z: 0.0, })
            .with(Velocity { dx: 0.0, dy: 0.0, dz: 0.0, })
            .with(Health { current: 100, maximum: 100, })
            .with(Sprite { texture_id: "player.png", width: 32.0, height: 32.0, color: Color::WHITE, })
            .id()
        
        # Create systems
        let systems: [Box⟨System⟩] = [
            Box::new(PlayerControlSystem),
            Box::new(MovementSystem),
            Box::new(CollisionSystem),
            Box::new(EventHandlerSystem),
            Box::new(RenderSystem::new())
        ]
        
        Self {
            world,
            systems,
            input: Input::new(),
            renderer: Renderer::new(),
        }
    }
    
    micro update(mut self, delta_time: f64) {
        # Update game time
        let mut game_time = self.world.get_resource_mut⟨GameTime⟩()
        game_time.delta_time = delta_time
        game_time.total_time += delta_time
        game_time.frame_count += 1
        
        # Run all systems
        for system in mut self.systems {
            system.run(mut self.world, delta_time)
        }
        
        # Clean up destroyed entities
        self.world.maintain()
    }
    
    micro spawn_enemy(mut self) {
        let x = random_range(0.0, 800.0)
        self.world.spawn()
            .with(Position { x, y: -50.0, z: 0.0, })
            .with(Velocity { dx: 0.0, dy: 50.0, dz: 1.0, })
            .with(Health { current: 30, maximum: 30, })
            .with(Sprite { texture_id: "enemy.png", width: 24.0, height: 24.0, color: Color::RED, })
            .id()
    }
    
    micro spawn_bullet(mut self, x: f64, y: f64) {
        self.world.spawn()
            .with(Position { x, y, z: 0.0, })
            .with(Velocity { dx: 0.0, dy: -200.0, dz: 0.0, })
            .with(Sprite { texture_id: "bullet.png", width: 4.0, height: 8.0, color: Color::YELLOW, })
            .with(Bullet)
    }
}

# Game main loop
micro main() {
    let mut game = SpaceShooterGame::new()
    let mut last_time = get_time()
    
    loop {
        let current_time = get_time()
        let delta_time = current_time - last_time
        last_time = current_time
        
        game.update(delta_time)
        
        # Control frame rate
        sleep(Duration::from_millis(16))  # ~60 FPS
    }
}
```

## Best Practices

1. **Component Design**: Keep components simple, containing only data, no logic
2. **System Responsibilities**: Each system should have a single responsibility, focusing on specific game logic
3. **Query Optimization**: Use appropriate query patterns, avoid unnecessary component access
4. **Memory Layout**: Consider cache-friendly data layouts, use SoA storage for hot components
5. **Parallelization**: Identify systems that can execute in parallel for better performance
6. **Event-Driven**: Use event systems to decouple communication between systems
7. **Resource Management**: Use global resources judiciously, avoid over-reliance

ECS architecture provides a flexible, high-performance solution for game development, especially suitable for complex game logic and large entity scenarios.
