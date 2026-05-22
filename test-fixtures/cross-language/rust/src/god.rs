// God module — high fan-in AND high fan-out → coupling-hotspot
use crate::types;
use crate::handlers;
use crate::models;
use crate::utils;
use crate::validators;
use crate::parsers;
use crate::converters;
use crate::formatters;
use crate::plugins;
use crate::adapters;
use crate::factories;
use crate::providers;
use crate::config;
use crate::storage;
use crate::network;

pub fn orchestrator() {
    types::get_type();
    handlers::dispatch();
    models::query();
    utils::helper();
    validators::check();
    parsers::parse();
    converters::convert();
    formatters::format();
    plugins::load();
    adapters::adapt();
    factories::create();
    providers::provide();
    config::load();
    storage::save();
    network::connect();
}
