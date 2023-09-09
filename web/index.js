const md_icon = `<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24"><path fill="currentColor" d="M20.56 18H3.44C2.65 18 2 17.37 2 16.59V7.41C2 6.63 2.65 6 3.44 6h17.12c.79 0 1.44.63 1.44 1.41v9.18c0 .78-.65 1.41-1.44 1.41M6.81 15.19v-3.66l1.92 2.35l1.92-2.35v3.66h1.93V8.81h-1.93l-1.92 2.35l-1.92-2.35H4.89v6.38h1.92M19.69 12h-1.92V8.81h-1.92V12h-1.93l2.89 3.28L19.69 12Z"/></svg>`;

let search_box = document.getElementById("search");
let search_btn = document.getElementById("search-btn");
let content = document.getElementById("doc");

const search = async () => {
    let res;
    if (search_box.value.endsWith(".md")) {
        res = await fetch("http://127.0.0.1:1440/notes/" + search_box.value);
    } else {
        res = await fetch("http://127.0.0.1:1440/notes/" + search_box.value + ".md");
    }
    
    if (res.status === 200) {
        let md = await res.text();
        content.innerHTML = md;
    } else {
        content.innerText = "404 Note Not Found";
    }
};

search_btn.onmousedown = search;
search_box.onkeydown = async (ev) => {
    if(ev.key === 'Enter') {
        await search();
    }
};

let list = document.getElementById("list");

fetch("http://127.0.0.1:1440/list?0:32").then(async res => {
    let json = await res.json();
    list.innerHTML = "";

    for (let i = 0; i < json.length; i++) {
        let el = document.createElement("li");
        el.innerHTML = md_icon + json[i];
        list.insertAdjacentElement("beforeend", el);
    }
});