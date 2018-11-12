use std::hash::Hash;

use cryptocurrency_kit::ethkey::Address;

use types::Validator;
use consensus::config::Config;
use consensus::validator::{Validators, ImplValidatorSet};

pub struct core<V> {
    config: Config,
    validators: V,
}

impl<ImplValidatorSet> core <ImplValidatorSet> {


}