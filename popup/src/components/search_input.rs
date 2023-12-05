use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub onchange: Callback<InputEvent>,
    pub value: AttrValue,
}
#[function_component(SearchInput)]
pub fn search_input(props: &Props) -> Html {
    html! {
                    <>
                        <label for="table-search" class="sr-only">{"Search"}</label>
                        <div class="relative mt-10 px-1" style="margin-bottom:1rem;">
                            <div class="absolute inset-y-0 rtl:inset-r-0 start-0 flex items-center ps-4 pointer-events-none">
                                <svg class="w-4 h-4 text-gray-500 dark:text-gray-400" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 20 20">
                                    <path stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="m19 19-4-4m0-7A7 7 0 1 1 1 8a7 7 0 0 1 14 0Z"/>
                                </svg>
                            </div>
                            <input type="text" id="table-search" class="block pt-2 ps-10 text-sm text-gray-900 border border-gray-300 rounded-lg w-64 bg-gray-50 focus:ring-blue-500 focus:border-blue-500 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500" placeholder="Search for accounts"
                             value={(props.value).clone()} oninput={props.onchange.clone()}/>
                        </div>
                    </>

    }
}
