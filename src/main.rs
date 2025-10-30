use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fs::{self, File};
use std::io::BufReader;

use anyhow::{Context as _, Result, anyhow};
use haste::demofile::DemoFile;
use haste::demostream::DemoStream;
use haste::entities::{self, DeltaHeader, Entity, deadlock_coord_from_cell, fkey_from_path};
use haste::fxhash;
use haste::parser::{Context, Parser, Visitor};

const DEADLOCK_PLAYERPAWN_ENTITY: u64 = fxhash::hash_bytes(b"CCitadelPlayerPawn");
const TICK_END: i32 = 10000;

#[derive(Debug)]
struct MyVisitor {
    positions: HashMap<u32, [f32; 3]>,
}

impl MyVisitor {
    fn new() -> Self {
        Self {
            positions: HashMap::new(),
        }
    }
}

impl Visitor for MyVisitor {
    fn on_entity(
        &mut self,
        _ctx: &Context,
        _delta_header: DeltaHeader,
        entity: &Entity,
    ) -> Result<()> {
        if _ctx.tick() > TICK_END {
            return Ok(());
        }
        println!("TICK: {}", _ctx.tick());

        if entity.serializer_name_heq(DEADLOCK_PLAYERPAWN_ENTITY) {
            let hero_id_fkey: u64 = fkey_from_path(&["m_nHeroID"]);
            let hero_id: u32 = entity.get_value(&hero_id_fkey).unwrap();
            println!("ID: {}", hero_id);
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    let filepath = args.get(1).context("usage: <executable> <filepath>")?;

    let file = File::open(filepath)?;
    let buf_reader = BufReader::new(file);
    let demo_file = DemoFile::start_reading(buf_reader)?;
    let visitor = MyVisitor::new();
    let mut parser = Parser::from_stream_with_visitor(demo_file, visitor)?;
    let ticks = parser.demo_stream_mut().total_ticks()?;
    println!("{} ", ticks);
    parser.run_to_end()?;
    // parser.run_to_tick(TICK_END)?;

    // Print collected field names

    Ok(())
}
