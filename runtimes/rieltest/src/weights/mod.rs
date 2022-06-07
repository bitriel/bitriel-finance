// Copyright 2022 SmallWorld Selendra.
// This file is part of Selendra.

// Selendra is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Selendra is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Selendra.  If not, see <http://www.gnu.org/licenses/>.

//! Expose the auto generated weight files.

pub mod block_weights;
pub mod extrinsic_weights;
pub mod module_session_manager;
pub mod paritydb_weights;
pub mod rocksdb_weights;

pub use block_weights::constants::BlockExecutionWeight;
pub use extrinsic_weights::constants::ExtrinsicBaseWeight;
pub use paritydb_weights::constants::ParityDbWeight;
pub use rocksdb_weights::constants::RocksDbWeight;
