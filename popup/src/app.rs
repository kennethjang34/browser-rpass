use yew::prelude::*;

#[function_component]
pub fn App() -> Html {
    let counter = use_state(|| 0);
    let onclick = {
        let counter = counter.clone();
        move |_: MouseEvent| {
            let value = *counter + 1;
            counter.set(value);
        }
    };

    html! {
        <div>
                <label for="account-search">{"Search for account:"}</label><br/>
                <input type="search" id="account-search" name="account-search"/>
            <button {onclick} >{ "Search" }</button>
            <table class="table table-bordered">
                <thead>
                  <tr>
                    <th>{ "ID" }</th>
                    <th>{ "Password" }</th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td id="account-id" class="pressable">
                        {"some_account_id" }
                    </td>
                    <td id="account-pw" class="pressable">
                        {"some_account_pw" }
                    </td>
                  </tr>
                </tbody>
            </table>
        </div>
    }
}
