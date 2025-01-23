use std::{marker::PhantomData, ops::Deref};

use rtrb::{Consumer, Producer, RingBuffer};

pub trait WiredThread {
    type Config;
    type Input;
    type Output;
    fn spawn_tokio_thread(config: Self::Config, input: Self::Input, output: Self::Output) -> Self
    where
        Self: Sized;
}

pub mod channel {
    use std::{
        marker::PhantomData,
        ops::{Deref, DerefMut},
    };

    use rtrb::RingBuffer;

    pub trait Origin: Send {}
    pub trait Destination: Send {}

    pub struct Sender<D, Type>
    where
        D: Destination,
    {
        sender: rtrb::Producer<Type>,
        target: PhantomData<D>,
    }

    pub struct Receiver<O, Type>
    where
        O: Origin,
    {
        receiver: rtrb::Consumer<Type>,
        target: PhantomData<O>,
    }

    impl<D: Destination, Type> Deref for Sender<D, Type> {
        type Target = rtrb::Producer<Type>;
        fn deref(&self) -> &Self::Target {
            &self.sender
        }
    }

    impl<O: Origin, Type> Deref for Receiver<O, Type> {
        type Target = rtrb::Consumer<Type>;
        fn deref(&self) -> &Self::Target {
            &self.receiver
        }
    }

    impl<D: Destination, Type> DerefMut for Sender<D, Type> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.sender
        }
    }

    impl<O: Origin, Type> DerefMut for Receiver<O, Type> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.receiver
        }
    }

    impl<D: Destination, Type> From<rtrb::Producer<Type>> for Sender<D, Type> {
        fn from(sender: rtrb::Producer<Type>) -> Self {
            Sender {
                sender,
                target: PhantomData,
            }
        }
    }

    impl<O: Origin, Type> From<rtrb::Consumer<Type>> for Receiver<O, Type> {
        fn from(sender: rtrb::Consumer<Type>) -> Self {
            Receiver {
                receiver: sender,
                target: PhantomData,
            }
        }
    }

    /// Work around foreign type limitaitons to implement [`From`] for tuples
    pub trait FromTuple<O> {
        fn from_tuple(o: O) -> Self;
    }

    pub trait IntoTuple<T> {
        fn into_tuple(self) -> T;
    }

    impl<O, T> IntoTuple<T> for O
    where
        T: FromTuple<O>,
    {
        fn into_tuple(self) -> T {
            T::from_tuple(self)
        }
    }

    impl<O: Origin, D: Destination, Type> FromTuple<(rtrb::Producer<Type>, rtrb::Consumer<Type>)>
        for (Sender<D, Type>, Receiver<O, Type>)
    {
        fn from_tuple(o: (rtrb::Producer<Type>, rtrb::Consumer<Type>)) -> Self {
            (Sender::from(o.0), Receiver::from(o.1))
        }
    }

    pub fn bounded<D: Destination, O: Origin, T: Default>(
        capacity: usize,
    ) -> (Sender<D, T>, Receiver<O, T>) {
        let (mut to, from): (Sender<D, T>, Receiver<O, T>) =
            RingBuffer::<T>::new(capacity).into_tuple();

        (to, from)
    }

    pub fn bounded_default<D: Destination, O: Origin, T: Default>() -> (Sender<D, T>, Receiver<O, T>)
    {
        bounded::<D, O, T>(100)
    }
}
