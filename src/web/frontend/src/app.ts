interface StatusResponse {
    status: string;
    project_name: string;
    features_passing: number;
    features_remaining: number;
}

async function fetchStatus() {
    const listElement = document.getElementById('project-list');
    if (!listElement) return;

    try {
        const response = await fetch('/api/status');
        const data: StatusResponse = await response.json();

        renderProjectCard(data);
    } catch (error) {
        console.error('Failed to fetch status:', error);
        listElement.innerHTML = '<div class="loading-state">ERROR: Link to control unit severed.</div>';
    }
}

function renderProjectCard(data: StatusResponse) {
    const listElement = document.getElementById('project-list');
    if (!listElement) return;

    const total = data.features_passing + data.features_remaining;
    const percentage = total > 0 ? Math.round((data.features_passing / total) * 100) : 0;

    listElement.innerHTML = `
        <article class="card">
            <div class="card-icon">
                <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path></svg>
            </div>
            <div class="card-content">
                <h2 class="card-title">${data.project_name}</h2>
                <div class="card-metadata">
                    <span>${data.features_passing} / ${total} FEATURES PASSING</span>
                    <span>${percentage}% COMPLETE</span>
                </div>
                <div class="card-tags">
                    <span class="tag ai">AI_AUTONOMOUS</span>
                    <span class="tag dev">RUST_BACKEND</span>
                    <span class="tag db">SQLITE_DB</span>
                </div>
            </div>
            <div class="card-link">
                <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path><polyline points="15 3 21 3 21 9"></polyline><line x1="10" y1="14" x2="21" y2="3"></line></svg>
            </div>
        </article>
    `;
}

async function runVibe() {
    const btn = document.getElementById('run-vibe-btn');
    if (!btn) return;

    btn.innerText = 'EXECUTING...';
    btn.style.opacity = '0.5';

    try {
        const response = await post('/api/run');
        console.log('Vibe loop triggered:', await response.text());
        setTimeout(() => {
            btn.innerText = 'INIT_VIBE';
            btn.style.opacity = '1';
            fetchStatus();
        }, 2000);
    } catch (error) {
        console.error('Failed to trigger vibe loop:', error);
        btn.innerText = 'FAILED';
    }
}

// Helper for POST requests
async function post(url: string) {
    return fetch(url, { method: 'POST' });
}

// Initial fetch
fetchStatus();

// Event Listeners
document.getElementById('refresh-btn')?.addEventListener('click', fetchStatus);
document.getElementById('run-vibe-btn')?.addEventListener('click', runVibe);
