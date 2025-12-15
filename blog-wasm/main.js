import init, { BlogApp } from "./pkg/blog_wasm.js";

let app;

function setPostsMessage(message) {
    const postsEl = document.getElementById("posts");
    postsEl.textContent = message;
}

async function refreshAuthStatus() {
    const statusEl = document.getElementById("auth-status");
    const isAuth = await app.isAuthenticated();
    statusEl.textContent = isAuth ? "Logged in" : "Not logged in";
}

async function loadPosts() {
    const postsEl = document.getElementById("posts");
    postsEl.textContent = "Loading...";

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
            title.textContent = p.title || "(no title)";

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
                delBtn.textContent = "Delete";
                delBtn.onclick = async () => {
                    try {
                        await app.deletePost(String(p.id));
                        await loadPosts();
                    } catch (e) {
                        console.error(e);
                        alert("Delete error");
                    }
                };

                actions.appendChild(delBtn);
                postEl.appendChild(actions);
            }

            postsEl.appendChild(postEl);
        }

        if (posts.length === 0) {
            setPostsMessage("No posts yet");
        }
    } catch (e) {
        console.error(e);
        postsEl.textContent = "Failed to load posts";
    }
}

async function main() {
    await init();
    app = new BlogApp("http://localhost:8080");

    // Registration
    document.getElementById("register-form").addEventListener("submit", async (e) => {
        e.preventDefault();

        const username = document.getElementById("reg-username").value.trim();
        const email = document.getElementById("reg-email").value.trim();
        const password = document.getElementById("reg-password").value.trim();

        if (!username || !email || !password) {
            alert("Please fill in all fields");
            return;
        }

        try {
            await app.register(username, email, password);
            await refreshAuthStatus();
            await loadPosts(); // load posts immediately after registration
            alert("Registration successful");
        } catch (err) {
            console.error(err);
            alert("Registration failed");
        }
    });

    // Login
    document.getElementById("login-form").addEventListener("submit", async (e) => {
        e.preventDefault();

        const email = document.getElementById("login-email").value.trim();
        const password = document.getElementById("login-password").value.trim();

        if (!email || !password) {
            alert("Please fill in all fields");
            return;
        }

        try {
            await app.login(email, password);
            await refreshAuthStatus();
            await loadPosts(); // load posts immediately after login
            alert("Login successful");
        } catch (err) {
            console.error(err);
            alert("Login failed");
        }
    });

    // Logout
    document.getElementById("logout-btn").addEventListener("click", async () => {
        try {
            app.logout();
            await refreshAuthStatus();
            setPostsMessage("You have logged out. Log in to see posts.");
        } catch (e) {
            console.error(e);
        }
    });

    // Create post
    document.getElementById("create-post-form").addEventListener("submit", async (e) => {
        e.preventDefault();

        const title = document.getElementById("post-title").value.trim();
        const content = document.getElementById("post-content").value.trim();

        if (!title || !content) {
            alert("Please fill in the title and content");
            return;
        }

        try {
            await app.createPost(title, content);
            document.getElementById("post-title").value = "";
            document.getElementById("post-content").value = "";
            await loadPosts();
        } catch (err) {
            console.error(err);
            alert("Failed to create post");
        }
    });

    // Initialize UI on page load
    await refreshAuthStatus();
    const isAuth = await app.isAuthenticated();
    if (isAuth) {
        await loadPosts();
    } else {
        setPostsMessage("Log in to see posts.");
    }
}

main().catch(console.error);
