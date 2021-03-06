use bevy::prelude::*;
use bevy::math::const_vec2;
use crate::yar::{Yar, YAR_BOUNDS, YarShootEvent, YarDiedEvent};
use crate::qotile::{Qotile, QOTILE_BOUNDS, QotileDiedEvent};
use crate::shield::{ShieldBlock, ShieldHealth, SHIELD_BLOCK_SPRITE_SIZE};
use crate::SCREEN_SIZE;
use crate::SCREEN_SCALE;
use crate::util;

const ZORLON_CANNON_SPEED:f32 = 6.0;
const ZORLON_CANNON_BOUNDS:Vec2 = const_vec2!([16.0*SCREEN_SCALE, 16.0*SCREEN_SCALE]);

pub struct SpawnZorlonCannonEvent;
pub struct DespawnZorlonCannonEvent;

pub struct ZorlonCannonPlugin;

impl Plugin for ZorlonCannonPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnZorlonCannonEvent>()
            .add_event::<DespawnZorlonCannonEvent>()
            .add_system(spawn)
            .add_system(despawn)
            .add_system(track)
            .add_system(shoot)
            .add_system(fly)
            .add_system(leave_world)
            .add_system(collide_yar)
            .add_system(collide_qotile)
            .add_system(collide_shield)
        ;
    }
}

#[derive(Component)]
pub struct ZorlonCannon
{
    launched: bool,
}

pub fn spawn(
    mut commands: Commands,
    mut spawn_event: EventReader<SpawnZorlonCannonEvent>,
    yar_query: Query<(&Transform, &Handle<TextureAtlas>), (With<Yar>, Without<ZorlonCannon>)>,
    zc_query: Query<&Transform, (With<ZorlonCannon>, Without<Yar>)>
) {
    if spawn_event.iter().next().is_none() || yar_query.is_empty() || !zc_query.is_empty() {
        return
    }

    let (yar_transform, texture_atlas_handle) = yar_query.single();

    let mut zorlon_transform = yar_transform.clone();
    zorlon_transform.translation.x = -SCREEN_SIZE.x / 2.0;

    commands
        .spawn_bundle(SpriteSheetBundle {
            sprite: TextureAtlasSprite { index: 23, ..default() },
            texture_atlas: texture_atlas_handle.clone(),
            transform: zorlon_transform.clone(),
            ..default()
        })
        .insert(ZorlonCannon {
            launched: false
        });
}

pub fn despawn(
    mut commands: Commands,
    mut despawn_event: EventReader<DespawnZorlonCannonEvent>,
    mut death_event: EventReader<YarDiedEvent>,
    query: Query<Entity, With<ZorlonCannon>>
) {
    if (despawn_event.iter().next().is_none() && death_event.iter().next().is_none()) || query.is_empty() {
        return;
    }

    let e = query.single();
    commands.entity(e).despawn();
}

pub fn track(
    yar_query: Query<&Transform, (With<Yar>, Without<ZorlonCannon>)>,
    mut zc_query: Query<(&mut Transform, &ZorlonCannon), Without<Yar>>
) {
    if yar_query.is_empty() || zc_query.is_empty() {
        return
    }

    let (mut zc_transform, zorlon_cannon) = zc_query.single_mut();
    if zorlon_cannon.launched {
        return
    }

    let yar_transform = yar_query.single();
    zc_transform.translation.y = yar_transform.translation.y;
}

pub fn shoot(
    mut shoot_event: EventReader<YarShootEvent>,
    mut zc_query: Query<&mut ZorlonCannon>
) {
    if shoot_event.iter().next().is_none() || zc_query.is_empty() {
        return
    }

    let mut zorlon_cannon = zc_query.single_mut();
    zorlon_cannon.launched = true;
}

pub fn fly(
    mut zc_query: Query<(&mut Transform, &ZorlonCannon)>
) {
    if zc_query.is_empty() {
        return
    }

    let (mut transform, zorlon_cannon) = zc_query.single_mut();
    if !zorlon_cannon.launched {
        return;
    }

    transform.translation.x += ZORLON_CANNON_SPEED;
}

pub fn leave_world(
    mut despawn_event: EventWriter<DespawnZorlonCannonEvent>,
    mut query: Query<&Transform, With<ZorlonCannon>>
) {
    if query.is_empty() {
        return;
    }

    let transform = query.single_mut();

    if util::is_offscreen( transform.translation ) {
        despawn_event.send(DespawnZorlonCannonEvent);
    }
}

pub fn collide_yar(
    mut death_event: EventWriter<YarDiedEvent>,
    mut despawn_event: EventWriter<DespawnZorlonCannonEvent>,
    yar_query: Query<&Transform, (With<Yar>, Without<ZorlonCannon>)>,
    zc_query: Query<(&Transform, &ZorlonCannon), Without<Yar>>
) {
    if yar_query.is_empty() || zc_query.is_empty() {
        return
    }

    let (zc_transform, zorlon_cannon) = zc_query.single();
    if !zorlon_cannon.launched {
        return
    }

    let yar_transform = yar_query.single();

    if util::intersect_rect(
        &yar_transform.translation,
        &YAR_BOUNDS,
        &zc_transform.translation,
        &ZORLON_CANNON_BOUNDS) {
        death_event.send(YarDiedEvent);
        despawn_event.send(DespawnZorlonCannonEvent);
    }
}

pub fn collide_qotile(
    mut death_event: EventWriter<QotileDiedEvent>,
    mut despawn_event: EventWriter<DespawnZorlonCannonEvent>,
    qotile_query: Query<&Transform, (With<Qotile>, Without<ZorlonCannon>)>,
    zc_query: Query<(&Transform, &ZorlonCannon), Without<Qotile>>
) {
    if qotile_query.is_empty() || zc_query.is_empty() {
        return
    }

    let (zc_transform, zorlon_cannon) = zc_query.single();
    if !zorlon_cannon.launched {
        return
    }

    let q_transform = qotile_query.single();

    if util::intersect_rect(
        &q_transform.translation,
        &QOTILE_BOUNDS,
        &zc_transform.translation,
        &ZORLON_CANNON_BOUNDS) {
        death_event.send(QotileDiedEvent);
        despawn_event.send(DespawnZorlonCannonEvent);
    }
}

pub fn collide_shield(
    mut despawn_event: EventWriter<DespawnZorlonCannonEvent>,
    mut shield_query: Query<(&Transform, &mut ShieldHealth), (With<ShieldBlock>, Without<ZorlonCannon>)>,
    zc_query: Query<(&Transform, &ZorlonCannon), Without<ShieldBlock>>
) {
    if shield_query.is_empty() || zc_query.is_empty() {
        return
    }

    let (zc_transform, zorlon_cannon) = zc_query.single();
    if !zorlon_cannon.launched {
        return
    }

    for (shield_transform, mut shield_health) in shield_query.iter_mut() {
        if util::intersect_rect(
            &shield_transform.translation,
            &YAR_BOUNDS,
            &zc_transform.translation,
            &SHIELD_BLOCK_SPRITE_SIZE) {
            shield_health.health -= 5;
            despawn_event.send(DespawnZorlonCannonEvent);
            return // Can only break one shield block at a time. Awful, really.
        }
    }
}