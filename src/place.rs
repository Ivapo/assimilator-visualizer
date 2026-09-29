//! Placement is the engine's code, reused (vis-001 §2.2.1): `NetworkData::from_config`
//! → `LinkGeometryIndex::from_network_config` → `interpolate(link, position, lane)`.

use std::collections::HashSet;

use anyhow::{Result, bail};
use assimilator_config::network::NetworkConfig;
use assimilator_config::types::LinkId;
use assimilator_core::network_data::NetworkData;
use assimilator_geometry::geo_util::LinkGeometryIndex;

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
