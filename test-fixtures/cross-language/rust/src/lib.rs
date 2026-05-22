// Module declarations
pub mod big;
pub mod god;
pub mod dup_a;
pub mod dup_b;
pub mod import_diverse;
pub mod error_swallow;
pub mod dangerous;

// Inline modules for barrel re-export detection
pub mod types { pub struct Foo; }
pub mod handlers { pub fn handle() {} }
pub mod models { pub struct Bar; }
pub mod utils { pub fn helper() {} }
pub mod validators { pub fn check() {} }
pub mod serializers { pub fn serialize() {} }
pub mod parsers { pub fn parse() {} }
pub mod converters { pub fn convert() {} }
pub mod formatters { pub fn format() {} }
pub mod middleware { pub fn middleware() {} }
pub mod plugins { pub fn load() {} }
pub mod adapters { pub fn adapt() {} }
pub mod factories { pub fn create() {} }
pub mod providers { pub fn provide() {} }

pub mod config { pub fn load() {} }
pub mod storage { pub fn save() {} }
pub mod network { pub fn connect() {} }

// Barrel re-exports — triggers reexport-entropy
pub use self::types::*;
pub use self::handlers::*;
pub use self::models::*;
pub use self::utils::*;
pub use self::validators::*;
pub use self::serializers::*;
pub use self::parsers::*;
pub use self::converters::*;
pub use self::formatters::*;
pub use self::middleware::*;
pub use self::plugins::*;
pub use self::adapters::*;
pub use self::factories::*;
pub use self::providers::*;

// orphan.rs is intentionally NOT declared here — dead-weight
