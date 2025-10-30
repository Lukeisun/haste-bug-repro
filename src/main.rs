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

#[derive(Debug)]
struct MyVisitor {
    positions: HashMap<u32, [f32; 3]>,
    tick_end: Option<i32>,
}

impl MyVisitor {
    fn new(tick_end: Option<i32>) -> Self {
        Self {
            positions: HashMap::new(),
            tick_end,
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
        if let Some(end) = self.tick_end {
            if _ctx.tick() > end {
                return Ok(());
            }
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
    let filepath = args
        .get(1)
        .context("usage: <executable> <filepath> --mode <end|tick> --tick-end <tick>")?;

    // Parse required mode argument
    let mode_pos = args
        .iter()
        .position(|arg| arg == "--mode")
        .context("--mode argument is required (use 'end' or 'tick')")?;
    let mode = args
        .get(mode_pos + 1)
        .context("--mode requires a value: 'end' or 'tick'")?;

    // Parse tick_end if in tick mode
    let tick_end = {
        let tick_end_pos = args
            .iter()
            .position(|arg| arg == "--tick-end")
            .context("--tick-end argument is required when using --mode tick")?;
        let tick_str = args
            .get(tick_end_pos + 1)
            .context("--tick-end requires a numeric value")?;
        Some(
            tick_str
                .parse::<i32>()
                .context("--tick-end value must be a valid integer")?,
        )
    };

    let file = File::open(filepath)?;
    let buf_reader = BufReader::new(file);
    let demo_file = DemoFile::start_reading(buf_reader)?;
    let visitor = MyVisitor::new(tick_end);
    let mut parser = Parser::from_stream_with_visitor(demo_file, visitor)?;
    let ticks = parser.demo_stream_mut().total_ticks()?;
    println!("Total ticks: {}", ticks);

    // Choose run mode based on mode argument
    match mode.as_str() {
        "tick" => {
            let end_tick = tick_end.unwrap();
            println!("Running to tick: {}", end_tick);
            parser.run_to_tick(end_tick)?;
        }
        "end" => {
            println!("Running to end");
            parser.run_to_end()?;
        }
        _ => unreachable!(),
    }

    Ok(())
}
