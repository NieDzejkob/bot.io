//! Small framework for commands that take input over multiple messages.
//!
//! The commands are modeled as [`genawaiter`] coroutines. The message sent by
//! the user is passed in as the resume value.

// Implementation note: genawaiter discards the first value passed to resume_with.
// (After all, where would you return it inside the generator?)
// We pass a dummy value to execute the code before the first yield.

use crate::prelude::*;
use genawaiter::{
    GeneratorState::*,
    sync::{Co, GenBoxed},
};
use serenity::{
    cache::CacheRwLock,
    http::{Http, CacheHttp},
};
use std::collections::HashMap;
use std::sync::Arc;

pub enum Event {
    Start,
    Message(MessageId, String),
    Reaction(MessageId, ReactionType),
}

/// Wait until the user responds with a message and return its contents. Ignore any other events.
pub async fn get_msg(co: &Co<(), Event>) -> String {
    loop {
        match co.yield_(()).await {
            Event::Message(_, contents) => return contents,
            _ => continue,
        }
    }
}

pub struct InteractiveCommand {
    pub generator: GenBoxed<(), Event>,
    pub abort_message: String,
}

pub struct InteractionState {
    command: InteractiveCommand,
    pending_abort: Option<InteractiveCommand>,
}

pub struct InteractionStates;

impl TypeMapKey for InteractionStates {
    type Value = HashMap<UserId, Arc<Mutex<InteractionState>>>;
}

pub fn handle_message(ctx: &mut Context, msg: &Message) -> CommandResult {
    let state =
        ctx.data.read()
            .get::<InteractionStates>().unwrap()
            .get(&msg.author.id)
            .map(Arc::clone);

    if let Some(state) = state {
        let mut state = state.lock();

        if let Some(next_command) = state.pending_abort.take() {
            match msg.content.to_lowercase().as_str() {
                "y" | "yes" => {
                    state.command = next_command;
                    state.command.generator.resume_with(Event::Start);
                }
                "n" | "no" => {}
                _ => {
                    state.pending_abort = Some(next_command);
                    msg.author.dm(&ctx, |m| m.content(format!(
                        "Please answer with `yes` or `no`. {}",
                        state.command.abort_message
                    ))).context("Send abort message").log_error();
                }
            }
        } else {
            if let Complete(()) = state.command.generator.resume_with(Event::Message(msg.id, msg.content.clone())) {
                ctx.data.write()
                    .get_mut::<InteractionStates>().unwrap()
                    .remove(&msg.author.id);
            }
        }

        Ok(())
    } else {
        crate::eval::handle_message(ctx, msg)
    }
}

impl InteractiveCommand {
    pub fn start(self, ctx: &mut Context, msg: &Message) {
        let mut lock = ctx.data.write();
        let entry = lock.get_mut::<InteractionStates>().unwrap()
            .entry(msg.author.id);

        use std::collections::hash_map::Entry::*;
        match entry {
            Occupied(e) => {
                let state = Arc::clone(e.get());
                let mut state = state.lock();
                drop(lock);
                state.pending_abort = Some(self);
                msg.author.dm(&ctx, |m| m.content(&state.command.abort_message))
                    .context("Send abort message").log_error();
            }
            Vacant(e) => {
                let state = Arc::new(Mutex::new(InteractionState {
                    command: self,
                    pending_abort: None,
                }));
                e.insert(Arc::clone(&state));
                let mut state = state.lock();
                drop(lock);
                state.command.generator.resume_with(Event::Start);
            }
        }
    }
}

// serenity::CacheAndHttp is nonexhaustive, and a tuple can't own the Arcs, so we need to roll our
// own struct for this
pub struct CacheAndHttp {
    cache: CacheRwLock,
    http: Arc<Http>,
}

impl CacheHttp for &CacheAndHttp {
    fn http(&self) -> &Http {
        &self.http
    }

    fn cache(&self) -> Option<&CacheRwLock> {
        Some(&self.cache)
    }
}
