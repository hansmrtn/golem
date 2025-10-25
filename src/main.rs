use bevy::{
    color::palettes::basic::{GRAY, GREEN, BLUE, PURPLE, WHITE},
    prelude::*,
};
use bevy_pancam::{PanCam, PanCamPlugin};
use noise::{NoiseFn, Perlin};
use rand::Rng;
use std::collections::HashMap;

// Perlin
const NOISE_SCALE: f64 = 10.3;
const TILE_SIZE: f32 = 12.;
const GRID_SIZE: f32 = 64.;

#[derive(Component)]
struct Player {
    pos: (i32, i32)
}

#[derive(Component)]
struct Tile {
    pos: (i32, i32),
    ttype: TileType
}

#[derive(Clone, Copy, PartialEq)]
enum TileType {
    Passable, 
    Unpassable
}

#[derive(Resource)]
struct TileMap {
    tiles: HashMap<(i32, i32), TileType>
}

impl TileMap {
    fn new() -> Self {
        Self {
            tiles: HashMap::new()
        }
    }
    
    fn is_passable(&self, pos: (i32, i32)) -> bool {
        self.tiles.get(&pos)
            .map(|t| *t == TileType::Passable)
            .unwrap_or(false)
    }
    
    fn insert(&mut self, pos: (i32, i32), ttype: TileType) {
        self.tiles.insert(pos, ttype);
    }
}

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, MeshPickingPlugin, PanCamPlugin::default()))
        .insert_resource(TileMap::new())
        .add_systems(Startup, setup)
        .add_systems(Update, move_player)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut tile_map: ResMut<TileMap>,
) {
    let hover_matl = materials.add(Color::from(WHITE));
    let default_matl = materials.add(Color::from(PURPLE));
    let player_matl = materials.add(Color::from(GREEN));
    
    commands.spawn((Camera2d, PanCam::default()));
    
    generate_world(&mut commands, &mut meshes, &mut materials, &mut tile_map);
    
    // Spawn player at the center of the tile grid
    let player_grid_pos = (GRID_SIZE as i32 / 2, GRID_SIZE as i32 / 2);
    let player_pos = Vec3::new(
        player_grid_pos.0 as f32 * TILE_SIZE,
        player_grid_pos.1 as f32 * TILE_SIZE,
        10.0, // Higher z-index to render above tiles
    );

    commands.spawn((
        Player { pos: player_grid_pos },
        Mesh2d(meshes.add(Rectangle::default())),
        MeshMaterial2d(player_matl),
        Transform::default()
            .with_scale(Vec3::splat(TILE_SIZE))
            .with_translation(player_pos),
    ));

    commands.spawn((
        Text::new("Move the light with WASD.\nThe camera will smoothly track the light."),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(12),
            ..default()
        },
    ));

}

fn move_player(
    mut player_query: Query<(&mut Player, &mut Transform)>,
    kb_input: Res<ButtonInput<KeyCode>>,
    tile_map: Res<TileMap>,
) {
    let Ok((mut player, mut transform)) = player_query.single_mut() else {
        return;
    };
    
    let mut direction = (0, 0);

    // Use just_pressed instead of pressed for single tile movement per key press
    if kb_input.just_pressed(KeyCode::KeyW) {
        direction.1 += 1;
    }

    if kb_input.just_pressed(KeyCode::KeyS) {
        direction.1 -= 1;
    }

    if kb_input.just_pressed(KeyCode::KeyA) {
        direction.0 -= 1;
    }

    if kb_input.just_pressed(KeyCode::KeyD) {
        direction.0 += 1;
    }

    // Move exactly one tile in the direction if valid
    if direction != (0, 0) {
        let new_pos = (player.pos.0 + direction.0, player.pos.1 + direction.1);
        
        // Check if the new position is passable
        if tile_map.is_passable(new_pos) {
            player.pos = new_pos;
            transform.translation = Vec3::new(
                new_pos.0 as f32 * TILE_SIZE,
                new_pos.1 as f32 * TILE_SIZE,
                transform.translation.z,
            );
        }
    }
}

fn generate_world(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<ColorMaterial>>,
    tile_map: &mut ResMut<TileMap>,
) {
    let mut rng = rand::thread_rng();
    let perlin = Perlin::new(12);
    
    let rock_matl = materials.add(Color::from(GRAY));
    let water_matl = materials.add(Color::from(BLUE));
    let ground_matl = materials.add(Color::from(PURPLE));
    let hover_matl = materials.add(Color::from(WHITE));
    
    for x in 0..GRID_SIZE as i32 {
        for y in 0..GRID_SIZE as i32 {
            let noise_val = perlin.get([x as f64 / NOISE_SCALE, y as f64 / NOISE_SCALE]);
            
            // Spawn ground tile
            let ground_material = ground_matl.clone();
            commands.spawn((
                Tile { pos: (x, y), ttype: TileType::Passable },
                Mesh2d(meshes.add(Rectangle::default())),
                MeshMaterial2d(ground_material.clone()),
                Transform::default()
                    .with_scale(Vec3::splat(TILE_SIZE))
                    .with_translation(Vec3::new(
                        x as f32 * TILE_SIZE,
                        y as f32 * TILE_SIZE,
                        0.0,
                    )),
                Pickable::default(),
            ))
            .observe(recolor_on::<Pointer<Over>>(hover_matl.clone()))
            .observe(recolor_on::<Pointer<Out>>(ground_material.clone()));
            
            // Track as passable in tile map
            tile_map.insert((x, y), TileType::Passable);
            
            // Spawn rock on top if noise is high enough
            if noise_val > 0.3 {
                let rock_material = rock_matl.clone();
                commands.spawn((
                    Tile { pos: (x, y), ttype: TileType::Unpassable },
                    Mesh2d(meshes.add(Rectangle::default())),
                    MeshMaterial2d(rock_material.clone()),
                    Transform::default()
                        .with_scale(Vec3::splat(TILE_SIZE))
                        .with_translation(Vec3::new(
                            x as f32 * TILE_SIZE,
                            y as f32 * TILE_SIZE,
                            1.0,
                        )),
                    Pickable::default(),
                ))
                .observe(recolor_on::<Pointer<Over>>(hover_matl.clone()))
                .observe(recolor_on::<Pointer<Out>>(rock_material.clone()));
                
                // Override with unpassable in tile map
                tile_map.insert((x, y), TileType::Unpassable);
            }


            let noise_val = perlin.get([x as f64 / NOISE_SCALE, y as f64 / NOISE_SCALE]);

            if noise_val > 0.8 {
                let rock_material = rock_matl.clone();
                commands.spawn((
                    Tile { pos: (x, y), ttype: TileType::Unpassable },
                    Mesh2d(meshes.add(Rectangle::default())),
                    MeshMaterial2d(water_matl.clone()),
                    Transform::default()
                        .with_scale(Vec3::splat(TILE_SIZE))
                        .with_translation(Vec3::new(
                            x as f32 * TILE_SIZE,
                            y as f32 * TILE_SIZE,
                            1.0,
                        )),
                    Pickable::default(),
                ))
                .observe(recolor_on::<Pointer<Over>>(hover_matl.clone()))
                .observe(recolor_on::<Pointer<Out>>(water_matl.clone()));
                
                // Override with unpassable in tile map
                tile_map.insert((x, y), TileType::Unpassable);
            }
        }
    }
}

fn recolor_on<E: EntityEvent>(
    new_material: Handle<ColorMaterial>,
) -> impl Fn(On<E>, Query<&mut MeshMaterial2d<ColorMaterial>>) {
    move |event, mut query| {
        if let Ok(mut material) = query.get_mut(event.event_target()) {
            material.0 = new_material.clone();
        }
    }
}
