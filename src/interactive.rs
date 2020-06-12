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
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum Event {
    Start,
    Message(MessageId, String),
    Reaction(Reaction),
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
    pub generator: GenBoxed<(), Event, Result<()>>,
    pub abort_message: Option<String>,
}

// Invariant: if command.abort_message is None, then pending_abort is also None.
pub struct InteractionState {
    command: InteractiveCommand,
    pending_abort: Option<InteractiveCommand>,
}

pub struct InteractionStates;

impl TypeMapKey for InteractionStates {
    type Value = HashMap<UserId, Arc<Mutex<InteractionState>>>;
}

fn get_state(ctx: &Context, user: UserId) -> Option<Arc<Mutex<InteractionState>>> {
    ctx.data.read()
        .get::<InteractionStates>().unwrap()
        .get(&user)
        .map(Arc::clone)
}

pub fn handle_message(ctx: &Context, msg: &Message) -> CommandResult {
    if let Some(state) = get_state(ctx, msg.author.id) {
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
                    msg.author.dm(ctx, |m| m.content(format!(
                        "Please answer with `yes` or `no`. {}",
                        state.command.abort_message.as_ref().unwrap()
                    ))).context("Send abort message").log_error();
                }
            }
        } else {
            if let Complete(status) = state.command.generator.resume_with(Event::Message(msg.id, msg.content.clone())) {
                status.log_error();
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

pub fn handle_reaction(ctx: &Context, reaction: Reaction) {
    if reaction.guild_id.is_some() {
        return;
    }

    if let Some(state) = get_state(ctx, reaction.user_id) {
        let mut state = state.lock();
        if state.pending_abort.is_none() {
            state.command.generator.resume_with(Event::Reaction(reaction));
        }
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
                let state = &mut *state; // allow partial borrows
                drop(lock);
                if let Some(ref abort_msg) = state.command.abort_message {
                    state.pending_abort = Some(self);
                    msg.author.dm(&ctx, |m| m.content(&abort_msg))
                        .context("Send abort message").log_error();
                } else {
                    state.command = self;
                    state.command.generator.resume_with(Event::Start);
                }
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
