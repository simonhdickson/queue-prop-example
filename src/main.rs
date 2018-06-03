#[macro_use] extern crate rand_derive;
#[macro_use] extern crate quickcheck;
extern crate rand;

use quickcheck::Gen;
use std::fmt::{self,Debug};
use rand::{ Rand};
use quickcheck::{Arbitrary,empty_shrinker};

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

trait Command<TActual, TModel> : Debug {
    fn pre(&self, &TActual, &TModel) -> bool;
    fn post(&self, &TActual, &TActual, &TModel, &TModel) -> bool;
    fn model(&self, TModel) -> TModel;
    fn actual(&mut self, TActual) -> TActual;
}

#[derive(Clone)]
struct QueueReset {}

impl Debug for QueueReset {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Reset")
    }
}

impl Command<Queue<i32>, usize> for QueueReset {
    fn pre(&self, _: &Queue<i32>, _: &usize) -> bool {
        true
    }
    fn post(&self, _: &Queue<i32>, actual: &Queue<i32>, _: &usize, model: &usize) -> bool {
        model == &actual.len()
    }
    fn model(&self, _: usize) -> usize {
        0
    }
    fn actual(&mut self, mut actual: Queue<i32>) -> Queue<i32> {
        actual.reset();
        actual
    }
}

#[derive(Clone)]
struct QueueGet<T> {
    actual_result: Option<T>
}

impl Debug for QueueGet<i32> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Get(actual_result:{:?})", self.actual_result)
    }
}

impl Command<Queue<i32>, usize> for QueueGet<i32> {
    fn pre(&self, _: &Queue<i32>, _: &usize) -> bool {
        true
    }
    fn post(&self, _: &Queue<i32>, actual: &Queue<i32>, old_model: &usize, model: &usize) -> bool {
        model == &actual.len() && (if old_model >= &1 { self.actual_result != None } else { true })
    }
    fn model(&self, model: usize) -> usize {
        if model == 0 { 0 } else { model - 1 }
    }
    fn actual(&mut self, mut actual: Queue<i32>) -> Queue<i32> {
        self.actual_result = actual.get();
        actual
    }
}

#[derive(Clone)]
struct QueuePush<T> {
    input_value: T
}

impl Debug for QueuePush<i32> {
fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "Push(input_value:{})", self.input_value)
        }
}


impl Command<Queue<i32>, usize> for QueuePush<i32> {
    fn pre(&self, _: &Queue<i32>, _: &usize) -> bool {
        true
    }
    fn post(&self, _: &Queue<i32>, actual: &Queue<i32>, _: &usize, model: &usize) -> bool {
        model == &actual.len()
    }
    fn model(&self, model: usize) -> usize {
        model + 1
    }
    fn actual(&mut self, mut actual: Queue<i32>) -> Queue<i32> {
        actual.push(self.input_value);
        actual
    }
}

#[derive(Rand,Clone,Debug)]
enum QueueCommand<T> where T: Rand {
    Get,
    Push(T),
    Reset
}

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

impl Into<Box<Command<Queue<i32>, usize>>> for QueueCommand<i32> {
    fn into(self) -> Box<Command<Queue<i32>, usize>> {
        match self {
            QueueCommand::Get       => Box::new(QueueGet{actual_result: None}),
            QueueCommand::Push(val) => Box::new(QueuePush{input_value: val}),
            QueueCommand::Reset     => Box::new(QueueReset{}),
        }
    }
}

fn main() {
}

#[cfg(test)]
quickcheck! {
    fn prop(commands: Vec<QueueCommand<i32>>) -> bool {
        let mut queue: Queue<i32> = Queue::new();
        let mut model = 0;
        let mut passed = true;
        for command in commands {
            let mut command: Box<Command<Queue<i32>, usize>> = command.into();
            if !(*command).pre(&queue, &model) {
                continue;
            }
            let old_queue = queue.clone();
            let old_model = model.clone();
            queue = (*command).actual(queue);
            model = (*command).model(model);
            if !(*command).post(&old_queue, &queue, &old_model, &model) {
                passed = false;
                break;
            }
        }

        assert!(passed, "oh no");
        
        true
    }
}
