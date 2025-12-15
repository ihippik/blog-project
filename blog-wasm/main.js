import init, { BlogApp } from "./pkg/blog_wasm.js";

let app;

function setPostsMessage(message) {
    const postsEl = document.getElementById("posts");
    postsEl.textContent = message;
}

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
        const posts = Array.isArray(res) ? res : Object.values(res);

        postsEl.innerHTML = "";

        for (const p of posts) {
            console.log(p);
            const postEl = document.createElement("div");
            postEl.className = "post";

            const header = document.createElement("div");
            header.className = "post-header";

            const title = document.createElement("div");
            title.className = "post-title";
            title.textContent = p.title || "(без заголовка)";

            const meta = document.createElement("div");
            meta.className = "post-meta";
            meta.textContent = p.created_at
                ? new Date(p.created_at).toLocaleString()
                : "";

            header.appendChild(title);
            header.appendChild(meta);
            postEl.appendChild(header);

            const content = document.createElement("div");
            content.className = "post-content";
            content.textContent = p.content || "";
            postEl.appendChild(content);

            if (p.id != null) {
                const actions = document.createElement("div");
                actions.className = "post-actions";

                const delBtn = document.createElement("button");
                delBtn.className = "btn-danger";
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

                actions.appendChild(delBtn);
                postEl.appendChild(actions);
            }

            postsEl.appendChild(postEl);
        }

        if (posts.length === 0) {
            setPostsMessage("Постов пока нет");
        }
    } catch (e) {
        console.error(e);
        postsEl.textContent = "Ошибка загрузки постов";
    }
}

async function main() {
    await init();
    app = new BlogApp("http://localhost:8080");

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
            await loadPosts(); // после регистрации сразу загружаем посты
            alert("Регистрация успешна");
        } catch (err) {
            console.error(err);
            alert("Ошибка регистрации");
        }
    });

    // Логин
    document.getElementById("login-form").addEventListener("submit", async (e) => {
        e.preventDefault();

        const email = document.getElementById("login-email").value.trim();
        const password = document.getElementById("login-password").value.trim();

        if (!email || !password) {
            alert("Заполни все поля");
            return;
        }

        try {
            await app.login(email, password);
            await refreshAuthStatus();
            await loadPosts(); // после логина загружаем посты
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
            setPostsMessage("Вы вышли из системы. Войдите, чтобы увидеть посты.");
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

    // Инициализация UI при заходе на страницу
    await refreshAuthStatus();
    const isAuth = await app.isAuthenticated();
    if (isAuth) {
        await loadPosts();
    } else {
        setPostsMessage("Войдите, чтобы увидеть посты.");
    }
}

main().catch(console.error);
