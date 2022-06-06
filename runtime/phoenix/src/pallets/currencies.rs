#![allow(unused_imports)]

use crate::*;

impl bholdus_currencies::Config for Runtime {
	type Event = Event;
	type MultiCurrency = Tokens;
	type WeightInfo = ();
}
