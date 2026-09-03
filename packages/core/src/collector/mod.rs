use std::sync::mpsc;
use crate::models::Observation;

pub type EventSender = mpsc::Sender<Observation>;
pub type EventReceiver = mpsc::Receiver<Observation>;
