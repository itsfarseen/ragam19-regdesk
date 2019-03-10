#[macro_export]

/**
 * Macro to add a constructor method named `build`
 * which takes a gtk::Builder and initialize the members
 * using it.
 * All members must have a corresponding widget in the UI file loaded into Builder,
 * such that the name of the member is the id of the widget.
 */
macro_rules! ui_struct {
    (struct $struct_name:ident {
        $($i:ident: $t:ty),+
    }) =>
    {
        struct $struct_name {
                $($i:$t,)+
        }
        impl $struct_name {
            fn build(builder: gtk::Builder) -> Self {
     //       Uncomment the below line to track when the widgets are destroyed.
     //       $(builder.get_object::<gtk::Widget>(stringify!($i)).unwrap().connect_destroy(|_| println!("destroying: {}::{}", stringify!($struct_name), stringify!($i)));)+
                Self {
                    $($i: builder.get_object(stringify!($i)).expect(stringify!($i)),)+
                }
            }
        }
    }
}
