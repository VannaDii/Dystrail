use crate::components::daisy_ui::foundation as f;

#[derive(f::Properties, PartialEq, Eq, Clone)]
pub struct TableProps {
    pub headers: Vec<f::AttrValue>,
    pub rows: Vec<Vec<f::AttrValue>>,
    #[prop_or_default]
    pub class: f::Classes,
}

#[f::function_component(Table)]
pub fn table(props: &TableProps) -> f::Html {
    let class = f::class_list(&["table", "table-zebra", "w-full"], &props.class);
    f::html! {
        <div class="overflow-x-auto">
            <table class={class} role="table">
                <thead>
                    <tr>
                        { for props.headers.iter().map(|head| f::html! { <th scope="col">{ head.clone() }</th> }) }
                    </tr>
                </thead>
                <tbody>
                    { for props.rows.iter().map(|row| f::html! {
                        <tr>
                            { for row.iter().map(|cell| f::html! { <td>{ cell.clone() }</td> }) }
                        </tr>
                    }) }
                </tbody>
            </table>
        </div>
    }
}
