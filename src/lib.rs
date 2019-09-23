// Queue Implementation
// This queue implementation is actually fundamentally wrong because it uses a
// Vector as the the underlying data store even though it doesn't work like a queue
// and our model doesn't detect this yet. At somepoint I'll add another property
// that shows why this is wrong, for now just pretend it works.

#[derive(Clone, Debug)]
pub struct Queue<T> {
    inner: Vec<T>,
}

impl<T> Queue<T> {
    pub fn new() -> Queue<T> {
        Queue { inner: Vec::new() }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn reset(&mut self) {
        for _ in 0..5 {
            self.inner.pop();
        }
    }

    pub fn get(&mut self) -> Option<T> {
        self.inner.pop()
    }

    pub fn push(&mut self, value: T) {
        self.inner.push(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::fmt::Debug;

    use quickcheck::{empty_shrinker, quickcheck, Arbitrary, Gen};
    use rand::Rand;
    use rand_derive::Rand;

    trait Model<TCommand, TActual>: Clone {
        fn pre(&self, command: &TCommand, actual: &TActual) -> bool;
        fn post(
            &self,
            command: &TCommand,
            actual_old: &TActual,
            actual_new: &TActual,
            model_old: &Self,
        ) -> bool;
        fn model(&mut self, command: &TCommand);
        fn actual(&mut self, command: &TCommand, actual: TActual) -> TActual;
    }

    fn confirm_model<TModel, TCommand, TActual: Clone + Debug>(
        mut model: TModel,
        commands: Vec<TCommand>,
        mut actual: TActual,
    ) -> bool
    where
        TModel: Model<TCommand, TActual>,
    {
        let mut passed = true;
        for command in commands {
            if !model.pre(&command, &actual) {
                continue;
            }
            let old_queue = actual.clone();
            let old_model = model.clone();
            actual = model.actual(&command, actual);
            model.model(&command);
            if !model.post(&command, &old_queue, &actual, &old_model) {
                passed = false;
                break;
            }
        }

        passed
    }

    #[derive(Rand, Clone, Debug)]
    enum QueueCommand<T>
    where
        T: Rand,
    {
        Get,
        Push(T),
        Reset,
    }

    #[derive(Clone, Default)]
    struct QueueModel {
        item_count: usize,
    }

    impl<T: Clone> Model<QueueCommand<T>, Queue<T>> for QueueModel
    where
        T: Rand,
    {
        fn pre(&self, command: &QueueCommand<T>, _: &Queue<T>) -> bool {
            match command {
                QueueCommand::Get => self.item_count != 0,
                QueueCommand::Push(_) => true,
                QueueCommand::Reset => true,
            }
        }

        fn post(&self, _: &QueueCommand<T>, _: &Queue<T>, actual: &Queue<T>, _: &Self) -> bool {
            self.item_count == actual.len()
        }

        fn model(&mut self, command: &QueueCommand<T>) {
            match command {
                QueueCommand::Get => self.item_count -= 1,
                QueueCommand::Push(_) => self.item_count += 1,
                QueueCommand::Reset => self.item_count = 0,
            }
        }

        fn actual(&mut self, command: &QueueCommand<T>, mut actual: Queue<T>) -> Queue<T> {
            match command {
                QueueCommand::Get => {
                    actual.get();
                }
                QueueCommand::Push(i) => actual.push(i.clone()),
                QueueCommand::Reset => actual.reset(),
            }
            actual
        }
    }

    impl<T: Arbitrary + Rand> Arbitrary for QueueCommand<T> {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            g.gen()
        }

        fn shrink(&self) -> Box<dyn Iterator<Item = QueueCommand<T>>> {
            match self {
                QueueCommand::Get => empty_shrinker(),
                QueueCommand::Reset => empty_shrinker(),
                QueueCommand::Push(x) => Box::new(x.shrink().map(QueueCommand::Push)),
            }
        }
    }

    quickcheck! {
        fn prop_i32(commands: Vec<QueueCommand<i32>>) -> bool {
            confirm_model(QueueModel::default(), commands, Queue::new())
        }
        fn prop_unit(commands: Vec<QueueCommand<()>>) -> bool {
            confirm_model(QueueModel::default(), commands, Queue::new())
        }
    }
}
