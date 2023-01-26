//!
//! @file lib.rs
//! @author Andrew Spaulding (Kasplat)
//! @brief (D)ynamically (I)nitialize a (S)tatic (Array).
//! @bug No known bugs.
//!

#[no_std]

///
/// Allows for a dynamically sized initialization of an array, capturing its size
/// in the identifier specified in the array type.
///
#[macro_export]
macro_rules! disarray {
    ( $(#[$meta:meta])* $scope:vis static $arr:ident: [$type:ty; $size:ident] = [
        $($items:expr),*
    ]; ) => {
        $scope const $size: usize = $crate::disarray!(@maybe_count $($items),*);
        $(#[$meta])* $scope static $arr: [$type; $size] = [ $($items),* ];
    };

    // Empty array len angers the compiler (idk).
    ( @maybe_count ) => { 0 };
    ( @maybe_count $($items:expr),+ ) => { [ $($crate::disarray!(@count $items)),* ].len() };

    // Make sure items are const.
    ( @count $item:expr ) => { 0 };
}
