use super::*;
#[allow(unused_imports)]
use log::*;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct MultiSelectProps {
    #[prop_or_default]
    items: Vec<AttrValue>,
    #[prop_or_default]
    pub id: AttrValue,
    #[prop_or_default]
    pub class: Classes,
    #[prop_or_default]
    pub style: AttrValue,
}
#[function_component(MultiSelect)]
pub fn multi_select(props: &MultiSelectProps) -> yew::Html {
    html! {
    <div class="w-full md:w-1/2 flex flex-col items-center mx-auto" style="height:50%;">
        <div class="w-full h-full">
            <div class="flex flex-col items-center relative" style="height:100%;">
                <div class="w-full">
                    <div class="my-2 p-1 flex border border-gray-200 bg-white rounded" style="height:70%;">
                        <div class="flex flex-auto flex-wrap">
                            <div class="flex justify-center items-center m-1 font-medium py-1 px-2 bg-white rounded-full text-teal-700 bg-teal-100 border border-teal-300 ">
                                <div class="text-xs font-normal leading-none max-w-full flex-initial">{"HTML"}</div>
                                <div class="flex flex-auto flex-row-reverse">
                                    <div>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-x cursor-pointer hover:text-teal-400 rounded-full w-4 h-4 ml-2">
                                            <line x1="18" y1="6" x2="6" y2="18"></line>
                                            <line x1="6" y1="6" x2="18" y2="18"></line>
                                        </svg>
                                    </div>
                                </div>
                            </div>
                            <div class="flex justify-center items-center m-1 font-medium py-1 px-2 bg-white rounded-full text-teal-700 bg-teal-100 border border-teal-300 ">
                                <div class="text-xs font-normal leading-none max-w-full flex-initial">{" Ruby" }</div>
                                <div class="flex flex-auto flex-row-reverse">
                                    <div>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-x cursor-pointer hover:text-teal-400 rounded-full w-4 h-4 ml-2">
                                            <line x1="18" y1="6" x2="6" y2="18"></line>
                                            <line x1="6" y1="6" x2="18" y2="18"></line>
                                        </svg>
                                    </div>
                                </div>
                            </div>
                            <div class="flex justify-center items-center m-1 font-medium py-1 px-2 bg-white rounded-full text-teal-700 bg-teal-100 border border-teal-300 ">
                                <div class="text-xs font-normal leading-none max-w-full flex-initial">{" Javascript" }</div>
                                <div class="flex flex-auto flex-row-reverse">
                                    <div>
                                        <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-x cursor-pointer hover:text-teal-400 rounded-full w-4 h-4 ml-2">
                                            <line x1="18" y1="6" x2="6" y2="18"></line>
                                            <line x1="6" y1="6" x2="18" y2="18"></line>
                                        </svg>
                                    </div>
                                </div>
                            </div>
                            <div class="flex-1">
                                <input placeholder="test...." class="bg-transparent p-1 px-2 appearance-none outline-none h-full w-full text-gray-800"/>
                            </div>
                        </div>
                        <div class="text-gray-300 w-8 py-1 pl-2 pr-1 border-l flex items-center border-gray-200">
                            <button class="cursor-pointer w-6 h-6 text-gray-600 outline-none focus:outline-none">
                                <svg xmlns="http://www.w3.org/2000/svg" width="100%" height="100%" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="feather feather-chevron-up w-4 h-4">
                                    <polyline points="18 15 12 9 6 15"></polyline>
                                </svg>
                            </button>
                        </div>
                    </div>
                </div>
                <Dropdown/>
            </div>
        </div>
    </div>
        }
}
