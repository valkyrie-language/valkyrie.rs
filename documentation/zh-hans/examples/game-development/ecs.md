# 实体组件系统 (ECS)

Valkyrie 提供了高性能的实体组件系统 (Entity Component System) 实现，这是现代游戏引擎的核心架构模式。ECS 将游戏对象分解为实体 (Entity)、组件 (Component) 和系统 (System)，实现了高度的模块化和性能优化。

## 核心概念

### 实体 (Entity)
实体是游戏世界中对象的唯一标识符，本身不包含数据或行为。

### 组件 (Component)
组件是纯数据结构，描述实体的属性和状态。

### 系统 (System)
系统包含游戏逻辑，对具有特定组件组合的实体进行操作。

## 基本ECS实现

```valkyrie
use valkyrie::ecs::*

# 定义组件
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

# 创建ECS世界
let world = World::new()

# 创建实体并添加组件
let player = world.spawn()
    .with(Position { x: 0.0, y: 0.0, z: 0.0, })
    .with(Velocity { dx: 0.0, dy: 0.0, dz: 0.0, })
    .with(Health { current: 100, maximum: 100, })
    .with(Sprite {
        texture_id: "player.png",
        width: 32.0,
        height: 32.0,
        color: Color::WHITE,
    })
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

## 系统实现

```valkyrie
# 移动系统
class MovementSystem;

imply MovementSystem: System {
    micro run(self, mut world: World, delta_time: f64) {
        # 查询所有具有Position和Velocity组件的实体
        loop (entity, (pos, vel)) in world.query⟨(mut Position, Velocity)⟩() {
            pos.x += vel.dx * delta_time
            pos.y += vel.dy * delta_time
            pos.z += vel.dz * delta_time
        }
    }
}

# 渲染系统
class RenderSystem {
    renderer: Renderer,
}

imply RenderSystem: System {
    micro run(self, world: World, _delta_time: f64) {
        # 查询所有可渲染的实体
        loop (entity, (pos, sprite)) in world.query⟨(Position, Sprite)⟩() {
            self.renderer.draw_sprite(
                sprite.texture_id,
                Vec2 { x: pos.x, y: pos.y, },
                Vec2 { x: sprite.width, y: sprite.height, },
                sprite.color,
            )
        }
    }
}

# 碰撞检测系统
class CollisionSystem;

imply CollisionSystem: System {
    micro run(self, mut world: World, _delta_time: f64) {
        let entities: [(Entity, Position, Sprite)] = 
            world.query⟨(Position, Sprite)⟩().collect()
        
        loop i in 0..entities.length {
            loop j in i + 1..entities.length {
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

## 高级ECS特性

### 组件标签和标记

```valkyrie
# 标记组件（无数据）
class Player;  # 标记实体为玩家
class Enemy;   # 标记实体为敌人
class Bullet;  # 标记实体为子弹
class Collectible;  # 标记实体为可收集物品

# 使用标记组件进行查询
class PlayerControlSystem;

imply PlayerControlSystem: System {
    micro run(self, mut world: World, input: Input) {
        # 只处理玩家实体
        loop (entity, (pos, vel)) in world.query⟨(mut Position, mut Velocity)⟩()
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

### 资源系统

```valkyrie
# 全局资源
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

# 在系统中使用资源
class ScoreSystem;

imply ScoreSystem: System {
    micro run(self, mut world: World, _delta_time: f64) {
        let mut score = world.get_resource_mut⟨Score⟩()
        let game_time = world.get_resource⟨GameTime⟩()
        
        # 每秒增加分数
        if game_time.frame_count % 60 == 0 {
            score.value += 10
            if score.value > score.high_score {
                score.high_score = score.value
            }
        }
    }
}
```

### 事件系统

```valkyrie
# 定义事件
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

# 事件处理系统
class EventHandlerSystem;

imply EventHandlerSystem: System {
    micro run(self, mut world: World, _delta_time: f64) {
        # 处理碰撞事件
        loop event in world.read_events⟨CollisionEvent⟩() {
            let e1_has_player = world.has_component⟨Player⟩(event.entity1)
            let e2_has_enemy = world.has_component⟨Enemy⟩(event.entity2)
            
            if e1_has_player && e2_has_enemy {
                # 玩家与敌人碰撞
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
        
        # 处理玩家死亡事件
        loop event in world.read_events⟨PlayerDeathEvent⟩() {
            print("Player died: {}", event.cause)
            world.despawn(event.player)
        }
    }
}
```

## 性能优化

### 组件存储优化

```valkyrie
# 使用SoA (Structure of Arrays) 存储
class PositionStorage {
    x_values: [f64],
    y_values: [f64],
    z_values: [f64],
    entities: [Entity],
}

# 批量处理
class BatchMovementSystem;

imply BatchMovementSystem: System {
    micro run(self, mut world: World, delta_time: f64) {
        # 获取所有位置和速度数据
        let positions = world.get_component_storage_mut⟨Position⟩()
        let velocities = world.get_component_storage⟨Velocity⟩()
        
        # 使用SIMD进行批量计算
        loop i in 0..positions.length {
            positions.x_values[i] += velocities.dx_values[i] * delta_time
            positions.y_values[i] += velocities.dy_values[i] * delta_time
            positions.z_values[i] += velocities.dz_values[i] * delta_time
        }
    }
}
```

### 并行系统执行

```valkyrie
use valkyrie::threading::*

# 并行系统调度器
class ParallelScheduler {
    thread_pool: ThreadPool,
}

imply ParallelScheduler {
    micro run_systems(self, mut world: World, systems: [Box⟨System⟩]) {
        # 分析系统依赖关系
        let dependency_graph = self.analyze_dependencies(systems)
        
        # 并行执行无依赖的系统
        let batches = self.create_execution_batches(dependency_graph)
        
        loop batch in batches {
            self.thread_pool.scope {
                loop system in batch {
                    %.spawn {
                        system.run(world, delta_time)
                    }
                }
            }
        }
    }
}
```

## 完整游戏示例

```valkyrie
# 简单的太空射击游戏
class SpaceShooterGame {
    world: World,
    systems: [Box⟨System⟩],
    input: Input,
    renderer: Renderer,
}

imply SpaceShooterGame {
    micro new() -> Self {
        let mut world = World::new()
        
        # 添加资源
        world.insert_resource(GameTime { total_time: 0.0, delta_time: 0.0, frame_count: 0, })
        world.insert_resource(Score { value: 0, high_score: 0, })
        
        # 创建玩家
        let player = world.spawn()
            .with(Position { x: 400.0, y: 500.0, z: 0.0, })
            .with(Velocity { dx: 0.0, dy: 0.0, dz: 0.0, })
            .with(Health { current: 100, maximum: 100, })
            .with(Sprite { texture_id: "player.png", width: 32.0, height: 32.0, color: Color::WHITE, })
            .with(Player)
            .id()
        
        # 创建系统
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
        # 更新游戏时间
        let mut game_time = self.world.get_resource_mut⟨GameTime⟩()
        game_time.delta_time = delta_time
        game_time.total_time += delta_time
        game_time.frame_count += 1
        
        # 运行所有系统
        loop system in mut self.systems {
            system.run(mut self.world, delta_time)
        }
        
        # 清理已销毁的实体
        self.world.maintain()
    }
    
    micro spawn_enemy(mut self) {
        let x = random_range(0.0, 800.0)
        self.world.spawn()
            .with(Position { x, y: -50.0, z: 0.0, })
            .with(Velocity { dx: 0.0, dy: 50.0, dz: 0.0, })
            .with(Health { current: 30, maximum: 30, })
            .with(Sprite { texture_id: "enemy.png", width: 24.0, height: 24.0, color: Color::RED, })
            .with(Enemy)
    }
    
    micro spawn_bullet(mut self, x: f64, y: f64) {
        self.world.spawn()
            .with(Position { x, y, z: 0.0, })
            .with(Velocity { dx: 0.0, dy: -200.0, dz: 0.0, })
            .with(Sprite { texture_id: "bullet.png", width: 4.0, height: 8.0, color: Color::YELLOW, })
            .with(Bullet)
    }
}

# 游戏主循环
micro main() {
    let mut game = SpaceShooterGame::new()
    let mut last_time = get_time()
    
    loop {
        let current_time = get_time()
        let delta_time = current_time - last_time
        last_time = current_time
        
        game.update(delta_time)
        
        # 控制帧率
        sleep(Duration::from_millis(16))  # ~60 FPS
    }
}
```

## 最佳实践

1. **组件设计**：保持组件简单，只包含数据，不包含逻辑
2. **系统职责**：每个系统应该有单一职责，专注于特定的游戏逻辑
3. **查询优化**：使用合适的查询模式，避免不必要的组件访问
4. **内存布局**：考虑缓存友好的数据布局，使用SoA存储热点组件
5. **并行化**：识别可以并行执行的系统，提高性能
6. **事件驱动**：使用事件系统解耦系统间的通信
7. **资源管理**：合理使用全局资源，避免过度依赖

ECS架构为游戏开发提供了灵活、高性能的解决方案，特别适合复杂的游戏逻辑和大量实体的场景。
