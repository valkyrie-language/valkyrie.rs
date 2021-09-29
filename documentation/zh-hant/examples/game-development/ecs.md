# 实體組件系統 (ECS)

Valkyrie 提供了高性能的实體組件系統 (Entity Component System) 實現，這是現代遊戲引擎的核心架構模式。ECS 將遊戲對象分解為实體 (Entity)、組件 (Component) 和系統 (System)，實現了高度的模組化和性能優化。

## 核心概念

### 实體 (Entity)
实體是遊戲世界中對象的唯一标识符，本身不包含數據或行為。

### 組件 (Component)
組件是纯數據結構，描述实體的屬性和狀態。

### 系統 (System)
系統包含遊戲邏輯，對具有特定組件組合的实體進行操作。

## 基本ECS實現

```valkyrie
use valkyrie::ecs::*

# 定義組件
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

# 創建ECS世界
let world = World::new()

# 創建实體並添加組件
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

## 系統實現

```valkyrie
# 移动系統
class MovementSystem;

imply MovementSystem: System {
    micro run(self, mut world: World, delta_time: f64) {
        # 查詢所有具有Position和Velocity組件的实體
        for (entity, (pos, vel)) in world.query⟨(mut Position, Velocity)⟩() {
            pos.x += vel.dx * delta_time
            pos.y += vel.dy * delta_time
            pos.z += vel.dz * delta_time
        }
    }
}

# 渲染系統
class RenderSystem {
    renderer: Renderer,
}

imply RenderSystem: System {
    micro run(self, world: World, _delta_time: f64) {
        # 查詢所有可渲染的实體
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

# 碰撞檢測系統
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

## 高級ECS特性

### 組件標籤和標記

```valkyrie
# 標記組件（無數據）
class Player;  # 標記实體為玩家
class Enemy;   # 標記实體為敌人
class Bullet;  # 標記实體為子弹
class Collectible;  # 標記实體為可收集物品

# 使用標記組件進行查詢
class PlayerControlSystem;

imply PlayerControlSystem: System {
    micro run(self, mut world: World, input: Input) {
        # 只處理玩家实體
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

### 資源系統

```valkyrie
# 全局資源
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

# 在系統中使用資源
class ScoreSystem;

imply ScoreSystem: System {
    micro run(self, mut world: World, _delta_time: f64) {
        let mut score = world.get_resource_mut⟨Score⟩()
        let game_time = world.get_resource⟨GameTime⟩()
        
        # 每秒增加分數
        if game_time.frame_count % 60 == 0 {
            score.value += 10
            if score.value > score.high_score {
                score.high_score = score.value
            }
        }
    }
}
```

### 事件系統

```valkyrie
# 定義事件
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

# 事件處理系統
class EventHandlerSystem;

imply EventHandlerSystem: System {
    micro run(self, mut world: World, _delta_time: f64) {
        # 處理碰撞事件
        loop event in world.read_events⟨CollisionEvent⟩() {
            let e1_has_player = world.has_component⟨Player⟩(event.entity1)
            let e2_has_enemy = world.has_component⟨Enemy⟩(event.entity2)
            
            if e1_has_player && e2_has_enemy {
                # 玩家與敌人碰撞
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
        
        # 處理玩家死亡事件
        loop event in world.read_events⟨PlayerDeathEvent⟩() {
            print("Player died: {}", event.cause)
            world.despawn(event.player)
        }
    }
}
```

## 性能優化

### 組件存儲優化

```valkyrie
# 使用SoA (Structure of Arrays) 存儲
class PositionStorage {
    x_values: [f64],
    y_values: [f64],
    z_values: [f64],
    entities: [Entity],
}

# 批量處理
class BatchMovementSystem;

imply BatchMovementSystem: System {
    micro run(self, mut world: World, delta_time: f64) {
        # 獲取所有位置和速度數據
        let positions = world.get_component_storage_mut⟨Position⟩()
        let velocities = world.get_component_storage⟨Velocity⟩()
        
        # 使用SIMD進行批量計算
        loop i in 0..positions.length {
            positions.x_values[i] += velocities.dx_values[i] * delta_time
            positions.y_values[i] += velocities.dy_values[i] * delta_time
            positions.z_values[i] += velocities.dz_values[i] * delta_time
        }
    }
}
```

### 並行系統執行

```valkyrie
use valkyrie::threading::*

# 並行系統調度器
class ParallelScheduler {
    thread_pool: ThreadPool,
}

imply ParallelScheduler {
    micro run_systems(self, mut world: World, systems: [Box⟨System⟩]) {
        # 分析系統依賴關係
        let dependency_graph = self.analyze_dependencies(systems)
        
        # 並行執行無依賴的系統
        let batches = self.create_execution_batches(dependency_graph)
        
        loop batch in batches {
            self.thread_pool.scope {
                loop system in batch {
                    $.spawn {
                        system.run(world, delta_time)
                    }
                }
            }
        }
    }
}
```

## 完整遊戲範例

```valkyrie
# 簡單的太空射击遊戲
class SpaceShooterGame {
    world: World,
    systems: [Box⟨System⟩],
    input: Input,
    renderer: Renderer,
}

imply SpaceShooterGame {
    micro new() -> Self {
        let mut world = World::new()
        
        # 添加資源
        world.insert_resource(GameTime { total_time: 0.0, delta_time: 0.0, frame_count: 0, })
        world.insert_resource(Score { value: 0, high_score: 0, })
        
        # 創建玩家
        let player = world.spawn()
            .with(Position { x: 400.0, y: 500.0, z: 0.0, })
            .with(Velocity { dx: 0.0, dy: 0.0, dz: 0.0, })
            .with(Health { current: 100, maximum: 100, })
            .with(Sprite { texture_id: "player.png", width: 32.0, height: 32.0, color: Color::WHITE, })
            .with(Player)
            .id()
        
        # 創建系統
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
        # 更新遊戲時間
        let mut game_time = self.world.get_resource_mut⟨GameTime⟩()
        game_time.delta_time = delta_time
        game_time.total_time += delta_time
        game_time.frame_count += 1
        
        # 運行所有系統
        loop system in mut self.systems {
            system.run(mut self.world, delta_time)
        }
        
        # 清理已销毁的实體
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

# 遊戲主循環
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

## 最佳實踐

1. **組件設計**：保持組件簡單，只包含數據，不包含邏輯
2. **系統职责**：每個系統應該有單一职责，专注于特定的遊戲邏輯
3. **查詢優化**：使用合適的查詢模式，避免不必要的組件訪問
4. **內存佈局**：考慮緩存友好的數據佈局，使用SoA存儲热點組件
5. **並行化**：识别可以並行執行的系統，提高性能
6. **事件驅動**：使用事件系統解耦系統間的通信
7. **資源管理**：合理使用全局資源，避免过度依賴

ECS架構為遊戲開發提供了靈活、高性能的解決案，特别适合複雜的遊戲邏輯和大量实體的場景。
