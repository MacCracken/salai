//! Entity picking via raycasting — click in viewport to select entities.
//!
//! Casts a ray from the camera through the click point and tests against
//! entity positions (sphere intersection) using [`hisab::geo`].

use hisab::geo::{Ray, Sphere, ray_sphere};
use hisab::{Mat4, Vec3};
use kiran::scene::Position;

/// Result of a pick operation.
#[derive(Debug, Clone)]
pub struct PickResult {
    /// The entity that was hit.
    pub entity: kiran::Entity,
    /// Distance from the camera to the hit point.
    pub distance: f32,
    /// World-space position of the entity.
    pub position: Vec3,
}

/// Parameters for a pick query.
pub struct PickQuery {
    pub camera_pos: Vec3,
    pub view_proj: Mat4,
    pub pixel_x: f32,
    pub pixel_y: f32,
    pub viewport_width: f32,
    pub viewport_height: f32,
    pub pick_radius: f32,
}

/// Cast a ray from camera through a viewport pixel and find the nearest entity hit.
#[must_use]
pub fn pick_entity(
    world: &kiran::World,
    entities: &[kiran::Entity],
    query: &PickQuery,
) -> Option<PickResult> {
    let ray = viewport_ray(
        query.camera_pos,
        query.view_proj,
        query.pixel_x,
        query.pixel_y,
        query.viewport_width,
        query.viewport_height,
    )?;

    let mut closest: Option<PickResult> = None;

    for &entity in entities {
        if !world.is_alive(entity) {
            continue;
        }
        let Some(pos) = world.get_component::<Position>(entity) else {
            continue;
        };

        let Ok(sphere) = Sphere::new(pos.0, query.pick_radius) else {
            continue;
        };

        if let Some(t) = ray_sphere(&ray, &sphere)
            && closest.as_ref().is_none_or(|c| t < c.distance)
        {
            closest = Some(PickResult {
                entity,
                distance: t,
                position: pos.0,
            });
        }
    }

    closest
}

/// Build a world-space ray from a viewport pixel click.
///
/// Uses [`hisab::transforms::screen_to_world_ray`] for the unprojection.
/// Returns `None` if the view-projection matrix is degenerate.
#[must_use]
#[inline]
fn viewport_ray(
    camera_pos: Vec3,
    view_proj: Mat4,
    pixel_x: f32,
    pixel_y: f32,
    viewport_width: f32,
    viewport_height: f32,
) -> Option<Ray> {
    if viewport_width < f32::EPSILON || viewport_height < f32::EPSILON {
        return None;
    }
    let det = view_proj.determinant();
    if det.abs() < f32::EPSILON {
        return None;
    }
    let inv = view_proj.inverse();

    let (_origin, direction) = hisab::transforms::screen_to_world_ray(
        pixel_x,
        pixel_y,
        inv,
        viewport_width,
        viewport_height,
    );

    Ray::new(camera_pos, direction).ok()
}

/// Convert viewport pixel coordinates to NDC.
///
/// `x`, `y` are pixel coordinates. `width`, `height` are viewport dimensions.
#[must_use]
#[inline]
pub fn pixel_to_ndc(x: f32, y: f32, width: f32, height: f32) -> (f32, f32) {
    if width < f32::EPSILON || height < f32::EPSILON {
        return (0.0, 0.0);
    }
    let ndc_x = (2.0 * x / width) - 1.0;
    let ndc_y = 1.0 - (2.0 * y / height); // Y is flipped
    (ndc_x, ndc_y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_sphere_hit() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0)).unwrap();
        let sphere = Sphere::new(Vec3::ZERO, 1.0).unwrap();

        let t = ray_sphere(&ray, &sphere).unwrap();
        assert!((t - 4.0).abs() < 0.01);
    }

    #[test]
    fn ray_sphere_miss() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0)).unwrap();
        let sphere = Sphere::new(Vec3::new(10.0, 0.0, 0.0), 1.0).unwrap();

        assert!(ray_sphere(&ray, &sphere).is_none());
    }

    #[test]
    fn ray_sphere_behind() {
        let ray = Ray::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, 1.0)).unwrap();
        let sphere = Sphere::new(Vec3::ZERO, 1.0).unwrap();

        assert!(ray_sphere(&ray, &sphere).is_none());
    }

    #[test]
    fn ray_sphere_inside() {
        let ray = Ray::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0)).unwrap();
        let sphere = Sphere::new(Vec3::ZERO, 5.0).unwrap();

        let t = ray_sphere(&ray, &sphere).unwrap();
        assert!(t > 0.0);
    }

    #[test]
    fn pixel_to_ndc_center() {
        let (x, y) = pixel_to_ndc(640.0, 360.0, 1280.0, 720.0);
        assert!(x.abs() < 0.01);
        assert!(y.abs() < 0.01);
    }

    #[test]
    fn pixel_to_ndc_corners() {
        let (x, y) = pixel_to_ndc(0.0, 0.0, 1280.0, 720.0);
        assert!((x - (-1.0)).abs() < 0.01);
        assert!((y - 1.0).abs() < 0.01);

        let (x, y) = pixel_to_ndc(1280.0, 720.0, 1280.0, 720.0);
        assert!((x - 1.0).abs() < 0.01);
        assert!((y - (-1.0)).abs() < 0.01);
    }

    fn query(camera_pos: Vec3, view_proj: Mat4, px: f32, py: f32, radius: f32) -> PickQuery {
        PickQuery {
            camera_pos,
            view_proj,
            pixel_x: px,
            pixel_y: py,
            viewport_width: 1280.0,
            viewport_height: 720.0,
            pick_radius: radius,
        }
    }

    #[test]
    fn pick_entity_finds_nearest() {
        let mut world = kiran::World::new();
        let near = world.spawn();
        let far = world.spawn();
        world
            .insert_component(near, Position(Vec3::new(0.0, 0.0, -3.0)))
            .unwrap();
        world
            .insert_component(far, Position(Vec3::new(0.0, 0.0, -10.0)))
            .unwrap();

        let cam = Vec3::new(0.0, 0.0, 0.0);
        let vp = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 100.0)
            * Mat4::look_at_rh(cam, Vec3::new(0.0, 0.0, -1.0), Vec3::Y);

        let hit = pick_entity(&world, &[near, far], &query(cam, vp, 640.0, 360.0, 1.0)).unwrap();
        assert_eq!(hit.entity, near);
    }

    #[test]
    fn pick_entity_no_hit() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world
            .insert_component(e, Position(Vec3::new(100.0, 0.0, 0.0)))
            .unwrap();

        let cam = Vec3::new(0.0, 0.0, 5.0);
        let vp = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0 / 9.0, 0.1, 100.0)
            * Mat4::look_at_rh(cam, Vec3::ZERO, Vec3::Y);

        assert!(pick_entity(&world, &[e], &query(cam, vp, 640.0, 360.0, 0.5)).is_none());
    }

    #[test]
    fn pick_entity_empty_world() {
        let world = kiran::World::new();
        assert!(
            pick_entity(
                &world,
                &[],
                &query(Vec3::ZERO, Mat4::IDENTITY, 0.0, 0.0, 1.0)
            )
            .is_none()
        );
    }

    #[test]
    fn pick_entity_skips_dead() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world.insert_component(e, Position(Vec3::ZERO)).unwrap();
        world.despawn(e).unwrap();

        assert!(
            pick_entity(
                &world,
                &[e],
                &query(Vec3::new(0.0, 0.0, 5.0), Mat4::IDENTITY, 50.0, 50.0, 1.0),
            )
            .is_none()
        );
    }

    #[test]
    fn pixel_to_ndc_zero_dimensions() {
        let (x, y) = pixel_to_ndc(100.0, 100.0, 0.0, 0.0);
        assert_eq!(x, 0.0);
        assert_eq!(y, 0.0);
    }

    #[test]
    fn pick_entity_degenerate_matrix() {
        let mut world = kiran::World::new();
        let e = world.spawn();
        world.insert_component(e, Position(Vec3::ZERO)).unwrap();

        assert!(
            pick_entity(
                &world,
                &[e],
                &query(Vec3::new(0.0, 0.0, 5.0), Mat4::ZERO, 50.0, 50.0, 1.0),
            )
            .is_none()
        );
    }
}
