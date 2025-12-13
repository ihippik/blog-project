import init, { BlogApp } from "./pkg/blog_wasm.js";

let app;

async function refreshAuthStatus() {
    const statusEl = document.getElementById("auth-status");
    const isAuth = await app.isAuthenticated();
    statusEl.textContent = isAuth ? "Залогинен" : "Не залогинен";
}

async function loadPosts() {
    const postsEl = document.getElementById("posts");
    postsEl.textContent = "Загрузка...";

    try {
        const res = await app.loadPosts();
        const posts = res; // JsValue уже объект/массив (serde-wasm-bindgen)

        if (!Array.isArray(posts)) {
            postsEl.textContent = "Неверный формат ответа";
            console.log("posts response:", posts);
            return;
        }

        postsEl.innerHTML = "";
        for (const p of posts) {
            const div = document.createElement("div");
            div.style.border = "1px solid #ccc";
            div.style.margin = "4px";
            div.style.padding = "4px";

            const title = document.createElement("strong");
            title.textContent = p.title || "(без заголовка)";
            div.appendChild(title);

            const content = document.createElement("p");
            content.textContent = p.content || "";
            div.appendChild(content);

            // Для упрощения: кнопка удаления по id
            if (p.id != null) {
                const delBtn = document.createElement("button");
                delBtn.textContent = "Удалить";
                delBtn.onclick = async () => {
                    try {
                        await app.deletePost(String(p.id));
                        await loadPosts();
                    } catch (e) {
                        console.error(e);
                        alert("Ошибка удаления");
                    }
                };
                div.appendChild(delBtn);
            }

            postsEl.appendChild(div);
        }
    } catch (e) {
        console.error(e);
        postsEl.textContent = "Ошибка загрузки постов";
    }
}

async function main() {
    // инициализация wasm-модуля
    await init();

    // адрес API – подставь свой
    app = new BlogApp("http://localhost:8080");

    // обработчики форм

    // Регистрация
    document.getElementById("register-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const username = document.getElementById("reg-username").value.trim();
        const email = document.getElementById("reg-email").value.trim();
        const password = document.getElementById("reg-password").value.trim();

        if (!username || !email || !password) {
            alert("Заполни все поля");
            return;
        }

        try {
            await app.register(username, email, password);
            await refreshAuthStatus();
            alert("Регистрация успешна");
        } catch (err) {
            console.error(err);
            alert("Ошибка регистрации");
        }
    });

    // Логин
    document.getElementById("login-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const username = document.getElementById("login-username").value.trim();
        const password = document.getElementById("login-password").value.trim();

        if (!username || !password) {
            alert("Заполни все поля");
            return;
        }

        try {
            await app.login(username, password);
            await refreshAuthStatus();
            alert("Вход успешен");
        } catch (err) {
            console.error(err);
            alert("Ошибка входа");
        }
    });

    // Logout
    document.getElementById("logout-btn").addEventListener("click", async () => {
        try {
            app.logout();
            await refreshAuthStatus();
        } catch (e) {
            console.error(e);
        }
    });

    // Создание поста
    document.getElementById("create-post-form").addEventListener("submit", async (e) => {
        e.preventDefault();
        const title = document.getElementById("post-title").value.trim();
        const content = document.getElementById("post-content").value.trim();

        if (!title || !content) {
            alert("Заполни заголовок и содержание");
            return;
        }

        try {
            await app.createPost(title, content);
            document.getElementById("post-title").value = "";
            document.getElementById("post-content").value = "";
            await loadPosts();
        } catch (err) {
            console.error(err);
            alert("Ошибка создания поста");
        }
    });

    await refreshAuthStatus();
    await loadPosts();
}

main().catch(console.error);
