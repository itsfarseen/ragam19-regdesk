#[macro_export]

macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
                move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
                move |$(clone!(@param $p),)+| $body
        }
    );
    ($($n:ident),+ => move |$p:ident: $t:ty| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
                move |$p: $t| $body
        }
    );
}
