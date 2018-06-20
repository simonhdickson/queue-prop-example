#[macro_use] extern crate rand_derive;
#[macro_use] extern crate quickcheck;
extern crate rand;

use quickcheck::Gen;
use rand::{ Rand};
use quickcheck::{Arbitrary,empty_shrinker};

// Generic Library Code

trait Command<TActual, TModel> {
    fn pre(&self, &TActual, &TModel) -> bool;
    fn post(&self, &TActual, &TActual, &TModel, &TModel) -> bool;
    fn model(&self, TModel) -> TModel;
    fn actual(&mut self, TActual) -> TActual;
}

#[cfg(test)]
fn confirm_model<TCommand, TActual: Clone, TModel: Clone>(commands: Vec<TCommand>, mut actual: TActual, mut model: TModel) -> bool
where TCommand: Into<Box<Command<TActual, TModel>>> {
    let mut passed = true;
    for command in commands {
        let mut command = command.into();
        if !(*command).pre(&actual, &model) {
            continue;
        }
        let old_queue = actual.clone();
        let old_model = model.clone();
        actual = (*command).actual(actual);
        model = (*command).model(model);
        if !(*command).post(&old_queue, &actual, &old_model, &model) {
            passed = false;
            break;
        }
    }

    passed
}

// Queue Implementation


#[derive(Clone)]
struct Queue<T> {
    inner: Vec<T>
}

impl<T> Queue<T> {
    fn new() -> Queue<T> {
        Queue {
            inner: Vec::new()
        }
    }

    fn len(&self) -> usize {
        self.inner.len()
    }

    fn reset(&mut self) {
        for _ in 0..5 {
            self.inner.pop();
        }
    }

    fn get(&mut self) -> Option<T> {
        self.inner.pop()
    }

    fn push(&mut self, value: T) {
        self.inner.push(value)
    }
}

// Possible actions that can be taken in this model

struct QueueReset {}

impl<T : Clone> Command<Queue<T>, usize> for QueueReset {
    fn pre(&self, _: &Queue<T>, _: &usize) -> bool {
        true
    }
    fn post(&self, _: &Queue<T>, actual: &Queue<T>, _: &usize, model: &usize) -> bool {
        model == &actual.len()
    }
    fn model(&self, _: usize) -> usize {
        0
    }
    fn actual(&mut self, mut actual: Queue<T>) -> Queue<T> {
        actual.reset();
        actual
    }
}

struct QueueGet<T> {
    actual_result: Option<T>
}

impl<T: Clone> Command<Queue<T>, usize> for QueueGet<T> {
    fn pre(&self, _: &Queue<T>, _: &usize) -> bool {
        true
    }
    fn post(&self, _: &Queue<T>, actual: &Queue<T>, old_model: &usize, model: &usize) -> bool {
        model == &actual.len() && (if old_model >= &1 { self.actual_result.is_some() } else { true })
    }
    fn model(&self, model: usize) -> usize {
        if model == 0 { 0 } else { model - 1 }
    }
    fn actual(&mut self, mut actual: Queue<T>) -> Queue<T> {
        self.actual_result = actual.get();
        actual
    }
}

struct QueuePush<T> {
    input_value: T
}

impl<T: Clone> Command<Queue<T>, usize> for QueuePush<T> {
    fn pre(&self, _: &Queue<T>, _: &usize) -> bool {
        true
    }
    fn post(&self, _: &Queue<T>, actual: &Queue<T>, _: &usize, model: &usize) -> bool {
        model == &actual.len()
    }
    fn model(&self, model: usize) -> usize {
        model + 1
    }
    fn actual(&mut self, mut actual: Queue<T>) -> Queue<T> {
        actual.push(self.input_value.clone());
        actual
    }
}

#[derive(Rand,Clone,Debug)]
enum QueueCommand<T> where T: Rand {
    Get,
    Push(T),
    Reset
}

// Arbitrary/Random implementations that allow the above commands to be
// randomly generated

impl<T: Arbitrary + Rand> Arbitrary for QueueCommand<T> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self { g.gen() }

    fn shrink(&self) -> Box<Iterator<Item=QueueCommand<T>>> {
        match self {
            QueueCommand::Get => empty_shrinker(),
            QueueCommand::Reset => empty_shrinker(),
            QueueCommand::Push(x) => Box::new(x.shrink().map(QueueCommand::Push)),
        }
    }
}

impl<T: Arbitrary + Rand, TModel> Into<Box<Command<Queue<T>, TModel>>> for QueueCommand<T>
where QueuePush<T>: Command<Queue<T>, TModel>, QueueReset: Command<Queue<T>, TModel>, QueueGet<T>: Command<Queue<T>, TModel> {
    fn into(self) -> Box<Command<Queue<T>, TModel>> {
        match self {
            QueueCommand::Get       => Box::new(QueueGet{actual_result: None}),
            QueueCommand::Push(val) => Box::new(QueuePush{input_value: val}),
            QueueCommand::Reset     => Box::new(QueueReset{}),
        }
    }
}

fn main() {
}

// We can run this model against any QueueCommand<T> where T implements the Rand trait
// I choose unit and i32 simply because they already implement it.

#[cfg(test)]
quickcheck! {
    fn prop_i32(commands: Vec<QueueCommand<i32>>) -> bool {
        confirm_model(commands, Queue::new(), 0)
    }
    fn prop_unit(commands: Vec<QueueCommand<()>>) -> bool {
        confirm_model(commands, Queue::new(), 0)
    }
}
