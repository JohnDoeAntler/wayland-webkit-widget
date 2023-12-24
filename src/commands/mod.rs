mod list;
mod kill;
mod create;
mod update;
mod show;
mod hide;
mod delete;
mod reload;
mod inspect;

pub mod handler;

pub trait Command {
    fn execute(&mut self) -> String;
}
