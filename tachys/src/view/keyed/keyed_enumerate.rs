use super::diff::{
    diff, unpack_moves, Diff, DiffOpAdd, DiffOpAddMode, DiffOpMove,
    DiffOpRemove, FxIndexSet, VecExt,
};
use crate::{
    html::attribute::Attribute,
    hydration::Cursor,
    renderer::{CastFrom, Renderer},
    ssr::StreamBuilder,
    view::{
        add_attr::AddAnyAttr, Mountable, Position, PositionState, Render,
        RenderHtml,
    },
};
use drain_filter_polyfill::VecExt as VecDrainFilterExt;
use reactive_graph::{signal::ArcWriteSignal, traits::Set};
use std::{hash::Hash, marker::PhantomData};

/// Creates a keyed list of views.
pub fn keyed_enumerate<T, I, K, KF, VF, V, Rndr>(
    items: I,
    key_fn: KF,
    view_fn: VF,
) -> KeyedEnumerate<T, I, K, KF, VF, V, Rndr>
where
    I: IntoIterator<Item = T>,
    K: Eq + Hash + 'static,
    KF: Fn(&T) -> K,
    V: Render<Rndr>,
    VF: Fn(usize, T) -> (ArcWriteSignal<usize>, V),
    Rndr: Renderer,
{
    KeyedEnumerate {
        items,
        key_fn,
        view_fn,
        rndr: PhantomData,
    }
}

/// A keyed list of views.
pub struct KeyedEnumerate<T, I, K, KF, VF, V, Rndr>
where
    I: IntoIterator<Item = T>,
    K: Eq + Hash + 'static,
    KF: Fn(&T) -> K,
    VF: Fn(usize, T) -> (ArcWriteSignal<usize>, V),
    Rndr: Renderer,
{
    items: I,
    key_fn: KF,
    view_fn: VF,
    rndr: PhantomData<Rndr>,
}

/// Retained view state for a keyed list.
pub struct KeyedEnumerateState<K, V, Rndr>
where
    K: Eq + Hash + 'static,
    V: Render<Rndr>,
    Rndr: Renderer,
{
    parent: Option<Rndr::Element>,
    marker: Rndr::Placeholder,
    hashed_items: FxIndexSet<K>,
    rendered_items: Vec<Option<V::State>>,
    index_items: Vec<Option<ArcWriteSignal<usize>>>,
}

impl<T, I, K, KF, VF, V, Rndr> Render<Rndr>
    for KeyedEnumerate<T, I, K, KF, VF, V, Rndr>
where
    I: IntoIterator<Item = T>,
    K: Eq + Hash + 'static,
    KF: Fn(&T) -> K,
    V: Render<Rndr>,
    VF: Fn(usize, T) -> (ArcWriteSignal<usize>, V),
    Rndr: Renderer,
{
    type State = KeyedEnumerateState<K, V, Rndr>;
    // TODO fallible state and try_build()/try_rebuild() here

    fn build(self) -> Self::State {
        let items = self.items.into_iter();
        let (capacity, _) = items.size_hint();
        let mut hashed_items =
            FxIndexSet::with_capacity_and_hasher(capacity, Default::default());
        let mut rendered_items = Vec::new();
        let mut index_items = Vec::new();
        for (index, item) in items.enumerate() {
            hashed_items.insert((self.key_fn)(&item));
            let (set_index, view) = (self.view_fn)(index, item);
            rendered_items.push(Some(view.build()));
            index_items.push(Some(set_index));
        }
        KeyedEnumerateState {
            parent: None,
            marker: Rndr::create_placeholder(),
            hashed_items,
            rendered_items,
            index_items,
        }
    }

    fn rebuild(self, state: &mut Self::State) {
        let KeyedEnumerateState {
            parent,
            marker,
            hashed_items,
            ref mut rendered_items,
            index_items,
        } = state;
        let new_items = self.items.into_iter();
        let (capacity, _) = new_items.size_hint();
        let mut new_hashed_items =
            FxIndexSet::with_capacity_and_hasher(capacity, Default::default());

        let mut items = Vec::new();
        for item in new_items {
            new_hashed_items.insert((self.key_fn)(&item));
            items.push(Some(item));
        }

        let cmds = diff(hashed_items, &new_hashed_items);

        apply_diff(
            parent
                .as_ref()
                .expect("Keyed list rebuilt before being mounted."),
            marker,
            cmds,
            rendered_items,
            &self.view_fn,
            items,
            index_items,
        );

        *hashed_items = new_hashed_items;
    }
}

impl<T, I, K, KF, VF, V, Rndr> AddAnyAttr<Rndr>
    for KeyedEnumerate<T, I, K, KF, VF, V, Rndr>
where
    I: IntoIterator<Item = T> + Send,
    K: Eq + Hash + 'static,
    KF: Fn(&T) -> K + Send,
    V: RenderHtml<Rndr>,
    V: 'static,
    VF: Fn(usize, T) -> (ArcWriteSignal<usize>, V) + Send + 'static,
    T: 'static,
    Rndr: Renderer,
{
    type Output<SomeNewAttr: Attribute<Rndr>> = KeyedEnumerate<
        T,
        I,
        K,
        KF,
        Box<
            dyn Fn(
                    usize,
                    T,
                ) -> (
                    ArcWriteSignal<usize>,
                    <V as AddAnyAttr<Rndr>>::Output<
                        SomeNewAttr::CloneableOwned,
                    >,
                ) + Send,
        >,
        V::Output<SomeNewAttr::CloneableOwned>,
        Rndr,
    >;

    fn add_any_attr<NewAttr: Attribute<Rndr>>(
        self,
        attr: NewAttr,
    ) -> Self::Output<NewAttr>
    where
        Self::Output<NewAttr>: RenderHtml<Rndr>,
    {
        let KeyedEnumerate {
            items,
            key_fn,
            view_fn,
            rndr,
        } = self;
        let attr = attr.into_cloneable_owned();
        KeyedEnumerate {
            items,
            key_fn,
            view_fn: Box::new(move |index, item| {
                let (index, view) = view_fn(index, item);
                (index, view.add_any_attr(attr.clone()))
            }),
            rndr,
        }
    }
}

impl<T, I, K, KF, VF, V, Rndr> RenderHtml<Rndr>
    for KeyedEnumerate<T, I, K, KF, VF, V, Rndr>
where
    I: IntoIterator<Item = T> + Send,
    K: Eq + Hash + 'static,
    KF: Fn(&T) -> K + Send,
    V: RenderHtml<Rndr> + 'static,
    VF: Fn(usize, T) -> (ArcWriteSignal<usize>, V) + Send + 'static,
    T: 'static,
    Rndr: Renderer,
{
    type AsyncOutput = Vec<V::AsyncOutput>; // TODO

    const MIN_LENGTH: usize = 0;

    fn dry_resolve(&mut self) {
        // TODO...
    }

    async fn resolve(self) -> Self::AsyncOutput {
        futures::future::join_all(self.items.into_iter().enumerate().map(
            |(index, item)| {
                let (_set_index, view) = (self.view_fn)(index, item);
                view.resolve()
            },
        ))
        .await
        .into_iter()
        .collect::<Vec<_>>()
    }

    fn to_html_with_buf(
        self,
        buf: &mut String,
        position: &mut Position,
        escape: bool,
    ) {
        for (index, item) in self.items.into_iter().enumerate() {
            let (_set_index, item) = (self.view_fn)(index, item);
            item.to_html_with_buf(buf, position, escape);
            *position = Position::NextChild;
        }
        buf.push_str("<!>");
    }

    fn to_html_async_with_buf<const OUT_OF_ORDER: bool>(
        self,
        buf: &mut StreamBuilder,
        position: &mut Position,
        escape: bool,
    ) {
        for (index, item) in self.items.into_iter().enumerate() {
            let (_set_index, item) = (self.view_fn)(index, item);
            item.to_html_async_with_buf::<OUT_OF_ORDER>(buf, position, escape);
            *position = Position::NextChild;
        }
        buf.push_sync("<!>");
    }

    fn hydrate<const FROM_SERVER: bool>(
        self,
        cursor: &Cursor<Rndr>,
        position: &PositionState,
    ) -> Self::State {
        // get parent and position
        let current = cursor.current();
        let parent = if position.get() == Position::FirstChild {
            current
        } else {
            Rndr::get_parent(&current)
                .expect("first child of keyed list has no parent")
        };
        let parent = Rndr::Element::cast_from(parent)
            .expect("parent of keyed list should be an element");

        // build list
        let items = self.items.into_iter();
        let (capacity, _) = items.size_hint();
        let mut hashed_items =
            FxIndexSet::with_capacity_and_hasher(capacity, Default::default());
        let mut rendered_items = Vec::new();
        let mut index_items = Vec::new();
        for (index, item) in items.enumerate() {
            hashed_items.insert((self.key_fn)(&item));
            let (set_index, view) = (self.view_fn)(index, item);
            let item = view.hydrate::<FROM_SERVER>(cursor, position);
            rendered_items.push(Some(item));
            index_items.push(Some(set_index));
        }
        let marker = cursor.next_placeholder(position);
        KeyedEnumerateState {
            parent: Some(parent),
            marker,
            hashed_items,
            rendered_items,
            index_items,
        }
    }
}

impl<K, V, Rndr> Mountable<Rndr> for KeyedEnumerateState<K, V, Rndr>
where
    K: Eq + Hash + 'static,
    V: Render<Rndr>,
    Rndr: Renderer,
{
    fn mount(&mut self, parent: &Rndr::Element, marker: Option<&Rndr::Node>) {
        self.parent = Some(parent.clone());
        for item in self.rendered_items.iter_mut().flatten() {
            item.mount(parent, marker);
        }
        self.marker.mount(parent, marker);
    }

    fn unmount(&mut self) {
        for item in self.rendered_items.iter_mut().flatten() {
            item.unmount();
        }
        self.marker.unmount();
    }

    fn insert_before_this(&self, child: &mut dyn Mountable<Rndr>) -> bool {
        self.rendered_items
            .first()
            .map(|n| n.insert_before_this(child))
            .unwrap_or_else(|| self.marker.insert_before_this(child))
    }
}

fn apply_diff<T, V, Rndr>(
    parent: &Rndr::Element,
    marker: &Rndr::Placeholder,
    diff: Diff,
    children: &mut Vec<Option<V::State>>,
    view_fn: impl Fn(usize, T) -> (ArcWriteSignal<usize>, V),
    mut items: Vec<Option<T>>,
    index_items: &mut Vec<Option<ArcWriteSignal<usize>>>,
) where
    V: Render<Rndr>,
    Rndr: Renderer,
{
    // The order of cmds needs to be:
    // 1. Clear
    // 2. Removals
    // 3. Move out
    // 4. Resize
    // 5. Move in
    // 6. Additions
    // 7. Removes holes
    if diff.clear {
        index_items.clear();

        for mut child in children.drain(0..) {
            child.unmount();
        }

        if diff.added.is_empty() {
            return;
        }
    }

    for DiffOpRemove { at } in &diff.removed {
        index_items[*at].take();

        let mut item_to_remove = children[*at].take().unwrap();

        item_to_remove.unmount();
    }

    let (move_cmds, add_cmds) = unpack_moves(&diff);

    let mut moved_children = vec![];
    let mut moved_index_items = vec![];
    for move_ in move_cmds.iter() {
        moved_children.push(children[move_.from].take());
        moved_index_items.push(index_items[move_.from].take());
    }

    children.resize_with(children.len() + diff.added.len(), || None);
    index_items.resize_with(index_items.len() + diff.added.len(), || None);

    for (i, DiffOpMove { to, .. }) in move_cmds
        .iter()
        .enumerate()
        .filter(|(_, move_)| !move_.move_in_dom)
    {
        children[*to] = moved_children[i].take();
        index_items[*to] =
            moved_index_items[i].take().inspect(|item| item.set(*to));
    }

    for (i, DiffOpMove { to, .. }) in move_cmds
        .into_iter()
        .enumerate()
        .filter(|(_, move_)| move_.move_in_dom)
    {
        let mut each_item = moved_children[i].take().unwrap();

        if let Some(Some(state)) = children.get_next_closest_mounted_sibling(to)
        {
            state.insert_before_this_or_marker(
                parent,
                &mut each_item,
                Some(marker.as_ref()),
            )
        } else {
            each_item.mount(parent, Some(marker.as_ref()));
        }

        children[to] = Some(each_item);
        index_items[to] =
            moved_index_items[i].take().inspect(|item| item.set(to));
    }

    for DiffOpAdd { at, mode } in add_cmds {
        let item = items[at].take().unwrap();
        let (set_index, item) = view_fn(at, item);
        let mut item = item.build();

        match mode {
            DiffOpAddMode::Normal => {
                if let Some(Some(state)) =
                    children.get_next_closest_mounted_sibling(at)
                {
                    state.insert_before_this_or_marker(
                        parent,
                        &mut item,
                        Some(marker.as_ref()),
                    )
                } else {
                    item.mount(parent, Some(marker.as_ref()));
                }
            }
            DiffOpAddMode::Append => {
                item.mount(parent, Some(marker.as_ref()));
            }
        }

        children[at] = Some(item);
        index_items[at] = Some(set_index);
    }

    #[allow(unstable_name_collisions)]
    children.drain_filter(|c| c.is_none());
    index_items.drain_filter(|c| c.is_none());
}
