use yew::prelude::*;
#[function_component(Home)] pub fn home()->Html{
    html!{
      <section class="panel">
        <img src="/static/img/logo.png" alt="Dystrail" style="image-rendering:pixelated;width:min(520px,80vw)"/>
        <div class="bar-wrap" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow="100"><div class="bar-fill" style="width:100%"/></div>
        <p class="muted">{"PRESS ANY KEY TO BEGIN"}</p>
        <div class="panel">
          <label for="code"><strong>{"Share Code"}</strong></label>
          <div class="controls">
            <input id="code" type="text" value="CL-PANTS42"/>
            <button>{"Start with Code"}</button>
          </div>
        </div>
        <div class="panel">
          <h3>{"Select Mode"}</h3>
          <div class="controls"><button>{"Classic"}</button><button>{"The Deep End"}</button></div>
        </div>
      </section>
    }
}