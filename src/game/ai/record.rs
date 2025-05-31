use std::fs::File;
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::game::ai::generation_stats::GenerationStatistics;

pub struct GenerationRecord {
    file: File,
    path: PathBuf
}

impl GenerationRecord {
    pub fn new() -> io::Result<Self> {
        let system_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        let filename = format!("generation-record-{}.csv", system_time.as_secs());
        let path = PathBuf::from(filename);
        let file = File::create(&path)?;
        let mut record = Self { file, path };

        // Write CSV header
        writeln!(record.file, "Generation,Score,Lines,Score P95,Lines P95,Score P50,Lines P50,Genome")?;
        Ok(record)
    }
    
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    pub fn add<const N: usize>(&mut self, stats: &GenerationStatistics<N>) -> io::Result<()> {
        writeln!(
            self.file,
            "{},{},{},{},{},{},{},\"{}\"",
            stats.id(),
            stats.max().result().score(),
            stats.max().result().lines(),
            stats.p95().result().score(),
            stats.p95().result().lines(),
            stats.median().result().score(),
            stats.median().result().lines(),
            stats.max().genome()
        )
    }


}