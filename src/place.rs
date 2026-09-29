//! Placement is the engine's code, reused (vis-001 §2.2.1): `NetworkData::from_config`
//! → `LinkGeometryIndex::from_network_config` → `interpolate(link, position, lane)`.
//! The junction movements and turn paths motion needs are the engine's too, and a point
//! on a turn path is `spatial_conflict`'s walk (§2.8.3).

use std::collections::HashSet;

use anyhow::{Result, bail};
use assimilator_config::network::MovementConfig;
use assimilator_config::network::NetworkConfig;
use assimilator_config::types::{LaneIdx, LinkId, NodeId};
use assimilator_core::network_data::{LaneTurnPath, NetworkData, RoutePair, TurnPathInfo};
use assimilator_core::spatial_conflict::{interpolate_heading, interpolate_pos};
use assimilator_geometry::geo_util::{
    LinkGeometryIndex, VehicleGeoPos, apply_perpendicular_offset,
};

use crate::fcd::Fcd;

/// A placed point: metres, and heading in degrees (0 = north, clockwise).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub x: f64,
    pub y: f64,
    pub heading: f64,
}

pub struct Placement {
    pub network: NetworkConfig,
    pub data: NetworkData,
    pub index: LinkGeometryIndex,
    ids: HashSet<String>,
}

impl Placement {
    pub fn new(network: NetworkConfig) -> Self {
        let data = NetworkData::from_config(&network);
        let index = LinkGeometryIndex::from_network_config(&network, &data);
        let ids = network.links.iter().map(|l| l.id.0.clone()).collect();
        Placement {
            network,
            data,
            index,
            ids,
        }
    }

    pub fn has_link(&self, link: &str) -> bool {
        self.ids.contains(link)
    }

    /// The engine's placement of one FCD row.
    pub fn place(&self, link: &str, position: f64, lane: u32) -> Option<Placed> {
        self.index
            .interpolate(link, position, lane)
            .map(|g| Placed {
                x: g.x,
                y: g.y,
                heading: g.heading,
            })
    }

    /// A point on the link's placed polyline with an explicit lateral offset.
    pub fn place_lateral(&self, link: &str, s: f64, lateral: f64) -> Option<Placed> {
        self.index
            .interpolate_with_lateral(link, s, lateral)
            .map(|g| Placed {
                x: g.x,
                y: g.y,
                heading: g.heading,
            })
    }

    /// The junction-trimmed centreline length FCD `position` is measured along.
    pub fn link_length(&self, link: &str) -> f64 {
        self.data.link_length(&LinkId(link.to_string()))
    }

    /// The link's lateral offset of `lane`'s centre (positive = right of travel).
    pub fn lane_offset(&self, link: &str, lane: u32) -> f64 {
        self.index.lane_offset(link, lane).unwrap_or(0.0)
    }

    /// The node a link ends at.
    pub fn to_node(&self, link: &str) -> Option<NodeId> {
        self.data.link_to_node(&LinkId(link.to_string())).cloned()
    }

    /// The engine's resolution of the route pair `a → b` at `n` (asm-045 §2.1).
    pub fn route_pair(&self, n: &NodeId, a: &str, b: &str) -> Option<RoutePair<'_>> {
        self.data
            .resolve_route_pair(n, &LinkId(a.to_string()), &LinkId(b.to_string()))
    }

    pub fn lane_turn_path(
        &self,
        n: &NodeId,
        m: &MovementConfig,
        from: u32,
        to: u32,
    ) -> Option<&LaneTurnPath> {
        self.data
            .lane_turn_path(n, &m.id, LaneIdx(from), LaneIdx(to))
    }

    /// The departure lane the engine picks on entry from `from`.
    pub fn resolve_departure_lane(&self, m: &MovementConfig, from: u32) -> u32 {
        self.data
            .resolve_departure_lane(&m.id, LaneIdx(from), &m.from_lanes, &m.to_lanes)
            .0
    }

    /// The movement's centreline turn path.
    pub fn turn_path(&self, n: &NodeId, m: &MovementConfig) -> Option<&TurnPathInfo> {
        self.data.turn_path(n, &m.id)
    }

    /// The point at arc length `s` on a turn path, moved `lateral` to the right of its
    /// tangent (`apply_perpendicular_offset`). The heading is converted from
    /// `interpolate_heading`'s radians anticlockwise from east to degrees clockwise from
    /// north (§2.8.3).
    pub fn turn_point(path: &[(f64, f64)], s: f64, lateral: f64) -> Placed {
        let (x, y) = interpolate_pos(path, s);
        let theta = interpolate_heading(path, s);
        let mut g = VehicleGeoPos {
            x,
            y,
            heading: (90.0 - theta.to_degrees()).rem_euclid(360.0),
            dir_x: theta.cos(),
            dir_y: theta.sin(),
        };
        apply_perpendicular_offset(&mut g, lateral);
        Placed {
            x: g.x,
            y: g.y,
            heading: g.heading,
        }
    }

    /// Place every row, before the first frame. An unknown `link_id` or a row the
    /// engine cannot place is an error.
    pub fn place_all(&self, fcd: &Fcd) -> Result<Vec<Placed>> {
        for name in &fcd.links {
            if !self.has_link(name) {
                bail!("unknown link id in FCD: {name:?} is not in the scenario's network");
            }
        }
        fcd.rows
            .iter()
            .map(|r| {
                let link = &fcd.links[r.link as usize];
                match self.place(link, r.position, r.lane) {
                    Some(p) => Ok(p),
                    None => bail!(
                        "cannot place FCD row: vehicle {} on link {link:?} lane {} at t={}",
                        r.vehicle_id,
                        r.lane,
                        r.time
                    ),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wrapped(a: f64, b: f64) -> f64 {
        let d = (a - b).rem_euclid(360.0);
        d.min(360.0 - d)
    }

    #[test]
    fn heading_conversion() {
        for (dx, dy, want) in [
            (0.0, 1.0, 0.0),
            (1.0, 0.0, 90.0),
            (0.0, -1.0, 180.0),
            (-1.0, 0.0, 270.0),
        ] {
            let p = Placement::turn_point(&[(0.0, 0.0), (dx, dy)], 0.5, 0.0);
            assert!(
                wrapped(p.heading, want) < 1e-9,
                "({dx},{dy}) gave {} not {want}",
                p.heading
            );
        }
    }

    /// Every per-lane and centreline turn path of the fixture: the point at 0, at each
    /// vertex's accumulated length and at `length` is that vertex, to 1e-9 m. Records how
    /// many first and last segments are shorter than 1e-9 m (§2.8.3).
    #[test]
    #[ignore = "needs scripts/fixture.sh"]
    fn turn_point_wiring() {
        let project = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("scratch/urban_grid");
        let pl = Placement::new(crate::inputs::load_network(&project, "baseline").unwrap());
        type Named = (String, Vec<(f64, f64)>, f64);
        let mut paths: Vec<Named> = Vec::new();
        for j in &pl.network.junctions {
            for m in &j.movements {
                for &from in &m.from_lanes {
                    for &to in &m.to_lanes {
                        if let Some(p) = pl.lane_turn_path(&j.node_id, m, from.0, to.0) {
                            paths.push((
                                format!("{} {}->{}", m.id.0, from.0, to.0),
                                p.path.clone(),
                                p.length,
                            ));
                        }
                    }
                }
                if let Some(p) = pl.turn_path(&j.node_id, m) {
                    paths.push((format!("{} centreline", m.id.0), p.path.clone(), p.length));
                }
            }
        }
        assert!(!paths.is_empty());
        let (mut worst, mut short_ends, mut points) = (0.0_f64, 0, 0);
        for (name, path, length) in &paths {
            let mut acc = 0.0;
            let mut checks = vec![(0.0, path[0])];
            for w in path.windows(2) {
                acc += ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt();
                checks.push((acc, w[1]));
            }
            checks.push((*length, *path.last().unwrap()));
            for (s, v) in checks {
                let p = Placement::turn_point(path, s, 0.0);
                let e = ((p.x - v.0).powi(2) + (p.y - v.1).powi(2)).sqrt();
                worst = worst.max(e);
                points += 1;
                assert!(e <= 1e-9, "{name}: s={s} is {e} m from its vertex");
            }
            let seg = |i: usize| {
                ((path[i + 1].0 - path[i].0).powi(2) + (path[i + 1].1 - path[i].1).powi(2)).sqrt()
            };
            let n = path.len() - 1;
            short_ends += (seg(0) < 1e-9) as usize;
            if n > 1 {
                short_ends += (seg(n - 1) < 1e-9) as usize;
            }
        }
        eprintln!(
            "turn_point wiring: {} paths, {points} points, worst {worst:.3e} m; \
             first/last segments shorter than 1e-9 m: {short_ends}",
            paths.len()
        );
    }
}
