use std::future::Future;
use std::pin::Pin;
use std::default::Default;
use pin_project_lite::pin_project;

use crate::stream::Stream;
use crate::task::{Context, Poll};

pin_project! {
    #[derive(Debug)]
    #[allow(missing_debug_implementations)]
    #[cfg(all(feature = "default", feature = "unstable"))]
    #[cfg_attr(feature = "docs", doc(cfg(unstable)))]
    pub struct PartitionMapFuture<S, F, A, B> {
        #[pin]
        stream: S,
        f: F,
        res: Option<(A, B)>,
    }
}

impl<S, F, A: Default, B: Default> PartitionMapFuture<S, F, A, B> {
    pub(super) fn new(stream: S, f: F) -> Self {
        Self {
            stream,
            f,
            res: Some((A::default(), B::default())),
        }
    }
}

impl<S, F, A, B, L, R> Future for PartitionMapFuture<S, F, A, B, L, R>
where
    S: Stream + Sized,
    F: FnMut(&S::Item) -> Either<L, R>,
    A: Default + Extend<L>,
    B: Default + Extend<R>,
{
    type Output = (A, B);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut this = self.project();

        loop {
            let next = futures_core::ready!(this.stream.as_mut().poll_next(cx));

            match next {
                Some(v) => {
                    let res = this.res.as_mut().unwrap();
                    match (this.f)(&v) {
                        Either::Left(l) => res.0.extend(l),
                        Either::Right(r) => res.1.extend(r),
                    };
                }
                None => return Poll::Ready(this.res.take().unwrap()),
            }
        }
    }
}
