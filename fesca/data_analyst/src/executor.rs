/*
physical plan -> MPC circuit translator
 */
use crate::physical_plan::PhysicalOp;
use anyhow::Result;
use log::info;
use computing_node::

// For now just log the physical plan. Later replace with circuit generation
pub fn execute(plan: &PhysicalOp) -> Result<()> {
    info!("Would execute MPC plan: {:#?}", plan);
    Ok(())
}