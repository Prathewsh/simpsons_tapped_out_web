import init, { create_game, homer_texture_manifest } from './pkg/simpsons_tapped_out_web.js';

async function main() {
  const loadingScreen = document.getElementById('loading-screen');
  const loadingBar    = document.getElementById('loading-bar');

  let progress = 0;
  const fakeProgress = setInterval(() => {
    progress = Math.min(progress + Math.random() * 5, 40);
    loadingBar.style.width = progress + '%';
  }, 120);

  try {
    await init();
    clearInterval(fakeProgress);

    const canvas = document.getElementById('game-canvas');
    fitCanvas(canvas);

    const game = create_game('game-canvas');

    // Load Homer's textures
    const manifest = JSON.parse(homer_texture_manifest());
    const total = manifest.length;
    let loaded = 0;

    document.querySelector('.loading-text').textContent = 'Loading Homer…';

    await Promise.all(manifest.map(({ key, path }) => {
      return new Promise((resolve, reject) => {
        const img = new Image();
        img.onload = () => {
          game.register_texture(key, img);
          loaded++;
          const p = 40 + (loaded / total) * 60;
          loadingBar.style.width = p + '%';
          resolve();
        };
        img.onerror = () => reject(new Error(`Failed to load: ${path}`));
        img.src = path;
      });
    }));

    loadingBar.style.width = '100%';

    // Bind global toast helper that Rust calls when a chore completes
    window.triggerRewardToast = (job, totalCash) => {
      const toast = document.getElementById('reward-notification-toast');
      const cashCount = document.getElementById('cash-count');
      
      let rewardText = "";
      if (job === "clean") rewardText = "Cleaned up Springfield! +$50, +10 XP";
      if (job === "play") rewardText = "Played Happy Little Elves! +$100, +25 XP";
      if (job === "drink") rewardText = "Drank beer at Moe's! +$150, +40 XP";
      
      document.getElementById('toast-message').textContent = rewardText;
      cashCount.textContent = totalCash;
      
      toast.classList.add('show');
      setTimeout(() => toast.classList.remove('show'), 3500);
    };

    const taskMenu = document.getElementById('task-selector-menu');
    let taskHomerTimer = null;

    // Resize handler
    window.addEventListener('resize', () => {
      fitCanvas(canvas);
      game.resize(canvas.width, canvas.height);
    });

    // Input forwarding
    canvas.addEventListener('mousemove', (e) => {
      const rect = canvas.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      game.on_mouse_move((e.clientX - rect.left) * dpr, (e.clientY - rect.top) * dpr);
    });
    
    canvas.addEventListener('click', (e) => {
      const rect = canvas.getBoundingClientRect();
      const dpr = window.devicePixelRatio || 1;
      const x = (e.clientX - rect.left) * dpr;
      const y = (e.clientY - rect.top) * dpr;

      // Check if we clicked Homer to trigger task dialog
      if (game.is_homer_clicked(x, y)) {
        // Toggle or display task popup menu above Homer
        const pos = game.get_homer_screen_pos();
        taskMenu.style.left = `${pos[0] + 30}px`;
        taskMenu.style.top = `${pos[1] - 80}px`;
        taskMenu.classList.remove('hidden');
        
        // Auto-close menu if we walk away
        if (taskHomerTimer) clearInterval(taskHomerTimer);
        taskHomerTimer = setInterval(() => {
          const freshPos = game.get_homer_screen_pos();
          taskMenu.style.left = `${freshPos[0] + 30}px`;
          taskMenu.style.top = `${freshPos[1] - 80}px`;
        }, 30);
      } else {
        // Clicked ground - hide menu and clear follow loop
        taskMenu.classList.add('hidden');
        if (taskHomerTimer) {
          clearInterval(taskHomerTimer);
          taskHomerTimer = null;
        }
        game.on_click(x, y);
      }
    });

    // Wire task choices buttons
    document.querySelectorAll('.task-item').forEach(item => {
      item.addEventListener('click', () => {
        const job = item.getAttribute('data-task');
        let duration = 6;
        if (job === "play") duration = 12;
        if (job === "drink") duration = 20;

        game.assign_homer_job(job, duration);
        
        // Hide menu
        taskMenu.classList.add('hidden');
        if (taskHomerTimer) {
          clearInterval(taskHomerTimer);
          taskHomerTimer = null;
        }
      });
    });

    // Hide loading screen
    setTimeout(() => loadingScreen.classList.add('hidden'), 300);

    // Main loop
    function loop(ts) {
      game.update(ts);
      requestAnimationFrame(loop);
    }
    requestAnimationFrame(loop);

  } catch (err) {
    clearInterval(fakeProgress);
    console.error('Failed to initialise game:', err);
    document.querySelector('.loading-text').textContent = 'Failed to load — check console';
  }
}

function fitCanvas(canvas) {
  canvas.width  = window.innerWidth  * window.devicePixelRatio;
  canvas.height = window.innerHeight * window.devicePixelRatio;
}

main();
