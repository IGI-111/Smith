#[macro_export]
macro_rules! delegate {
    ( $fld:ident : ) => {
    };

    ( $fld:ident : $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ) => {
        fn $fcn ( &self, $( $a : $at ),* ) -> $r {
            (self.$fld).$fcn( $( $a ),* )
        }
    };

    ( $fld:ident : $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty, $($rest:tt)* ) => {
        fn $fcn ( &self, $( $a : $at ),* ) -> $r {
            (self.$fld).$fcn( $( $a ),* )
        }
        delegate!($fld : $($rest)*);
    };

    ( $fld:ident : pub $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ) => {
        pub fn $fcn ( &self, $( $a : $at ),* ) -> $r {
            (self.$fld).$fcn( $( $a ),* )
        }
    };

    ( $fld:ident : pub $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty, $($rest:tt)* ) => {
        pub fn $fcn ( &self, $( $a : $at ),* ) -> $r {
            (self.$fld).$fcn( $( $a ),* )
        }
        delegate!($fld : $($rest)*);
    };

    ( $fld:ident : mut $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ) => {
        fn $fcn ( &mut self, $( $a : $at ),* ) -> $r {
            (self.$fld).$fcn( $( $a ),* )
        }
    };

    ( $fld:ident : mut $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty, $($rest:tt)* ) => {
        fn $fcn ( &mut self, $( $a : $at ),* ) -> $r {
            (self.$fld).$fcn( $( $a ),* )
        }
        delegate!($fld : $($rest)*);
    };

    ( $fld:ident : pub mut $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty ) => {
        pub fn $fcn ( &mut self, $( $a : $at ),* ) -> $r {
            (self.$fld).$fcn( $( $a ),* )
        }
    };

    ( $fld:ident : pub mut $fcn:ident ( $( $a:ident : $at:ty ),* ) -> $r:ty, $($rest:tt)* ) => {
        pub fn $fcn ( &mut self, $( $a : $at ),* ) -> $r {
            (self.$fld).$fcn( $( $a ),* )
        }
        delegate!($fld : $($rest)*);
    };

}
