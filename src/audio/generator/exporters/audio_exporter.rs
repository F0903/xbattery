use std::path::Path;

use crate::AppResult;

pub(crate) trait AudioExporter {
    fn export(&self, path: &Path, sample_rate: u32, samples: &[i16]) -> AppResult<()>;
}
