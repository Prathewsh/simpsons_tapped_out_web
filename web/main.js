import init, { create_game, homer_texture_manifest, bart_texture_manifest } from './pkg/simpsons_tapped_out_web.js';

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

    // Load textures
    const homerManifest = JSON.parse(homer_texture_manifest());
    const bartManifest = JSON.parse(bart_texture_manifest());
    const manifest = [...homerManifest, ...bartManifest];
    const total = manifest.length;
    let loaded = 0;

    document.querySelector('.loading-text').textContent = 'Loading Springfield…';

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

    let donutCount = 10; // starting donuts

    // Donut rewards per job
    const donutRewards = {
      clean: 1, play: 2, drink: 3,
      skateboard: 1, slingshot: 2, play_simulator: 3,
    };

    // Bind global toast helper that Rust calls when a chore completes
    window.triggerRewardToast = (job, totalCash) => {
      const toast    = document.getElementById('reward-notification-toast');
      const cashEl   = document.getElementById('cash-count');
      const donutEl  = document.getElementById('donut-count');

      // Update donut count
      const donuts = donutRewards[job] ?? 1;
      donutCount += donuts;
      donutEl.textContent = donutCount;

      let rewardText = '';
      if (job === 'clean')          rewardText = `Cleaned up Springfield! +$50, +${donuts}🍩`;
      if (job === 'play')           rewardText = `Played Happy Little Elves! +$100, +${donuts}🍩`;
      if (job === 'drink')          rewardText = `Drank beer at Moe's! +$150, +${donuts}🍩`;
      if (job === 'skateboard')     rewardText = `Rode Skateboard! +$60, +${donuts}🍩`;
      if (job === 'slingshot')      rewardText = `Shot Slingshot! +$120, +${donuts}🍩`;
      if (job === 'play_simulator') rewardText = `Played Yard Work Simulator! +$180, +${donuts}🍩`;

      document.getElementById('toast-message').textContent = rewardText;
      cashEl.textContent = totalCash;

      toast.classList.add('show');
      setTimeout(() => toast.classList.remove('show'), 3500);
    };


    const taskMenu = document.getElementById('task-selector-menu');
    const bartTaskMenu = document.getElementById('bart-task-selector-menu');
    let taskHomerTimer = null;
    let taskBartTimer = null;

    // Menu state UI
    const tapToStart = document.getElementById('tap-to-start');
    const gameHud    = document.getElementById('game-hud');

    function enterPlaying() {
      tapToStart.style.display = 'none';
      gameHud.classList.remove('hidden');
      // Notify Rust to switch from Menu → Playing
      game.on_click(0, 0);
    }

    tapToStart.addEventListener('click', enterPlaying);

    // Quest book: toggle character tray expand-right
    const characterTray = document.getElementById('character-tray');
    const questBookBtn  = document.getElementById('quest-book-btn');
    questBookBtn.addEventListener('click', (e) => {
      e.stopPropagation();
      const isOpen = characterTray.classList.toggle('open');
      questBookBtn.classList.toggle('tray-open', isOpen);
    });
    // Close tray when clicking elsewhere
    document.addEventListener('click', () => {
      characterTray.classList.remove('open');
      questBookBtn.classList.remove('tray-open');
    });
    characterTray.addEventListener('click', (e) => e.stopPropagation());


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
        bartTaskMenu.classList.add('hidden');
        if (taskBartTimer) clearInterval(taskBartTimer);

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
      } else if (game.is_bart_clicked(x, y)) {
        taskMenu.classList.add('hidden');
        if (taskHomerTimer) clearInterval(taskHomerTimer);

        // Toggle or display task popup menu above Bart
        const pos = game.get_bart_screen_pos();
        bartTaskMenu.style.left = `${pos[0] + 30}px`;
        bartTaskMenu.style.top = `${pos[1] - 80}px`;
        bartTaskMenu.classList.remove('hidden');

        if (taskBartTimer) clearInterval(taskBartTimer);
        taskBartTimer = setInterval(() => {
          const freshPos = game.get_bart_screen_pos();
          bartTaskMenu.style.left = `${freshPos[0] + 30}px`;
          bartTaskMenu.style.top = `${freshPos[1] - 80}px`;
        }, 30);
      } else {
        // Clicked ground - hide menu and clear follow loop
        taskMenu.classList.add('hidden');
        bartTaskMenu.classList.add('hidden');
        if (taskHomerTimer) {
          clearInterval(taskHomerTimer);
          taskHomerTimer = null;
        }
        if (taskBartTimer) {
          clearInterval(taskBartTimer);
          taskBartTimer = null;
        }
        game.on_click(x, y);
      }
    });

    // Wire task choices buttons
    document.querySelectorAll('#task-selector-menu .task-item').forEach(item => {
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

    // Wire Bart task choices buttons
    document.querySelectorAll('#bart-task-selector-menu .task-item').forEach(item => {
      item.addEventListener('click', () => {
        const job = item.getAttribute('data-task');
        let duration = 6;
        if (job === "slingshot") duration = 12;
        if (job === "play_simulator") duration = 20;

        game.assign_bart_job(job, duration);
        
        bartTaskMenu.classList.add('hidden');
        if (taskBartTimer) {
          clearInterval(taskBartTimer);
          taskBartTimer = null;
        }
      });
    });

    // Sidebar card clicks to select character and open task menu
    document.getElementById('char-card-homer').addEventListener('click', (e) => {
      e.stopPropagation();
      bartTaskMenu.classList.add('hidden');
      if (taskBartTimer) clearInterval(taskBartTimer);

      const pos = game.get_homer_screen_pos();
      taskMenu.style.left = `${pos[0] + 30}px`;
      taskMenu.style.top = `${pos[1] - 80}px`;
      taskMenu.classList.remove('hidden');

      if (taskHomerTimer) clearInterval(taskHomerTimer);
      taskHomerTimer = setInterval(() => {
        const freshPos = game.get_homer_screen_pos();
        taskMenu.style.left = `${freshPos[0] + 30}px`;
        taskMenu.style.top = `${freshPos[1] - 80}px`;
      }, 30);
    });

    document.getElementById('char-card-bart').addEventListener('click', (e) => {
      e.stopPropagation();
      taskMenu.classList.add('hidden');
      if (taskHomerTimer) clearInterval(taskHomerTimer);

      const pos = game.get_bart_screen_pos();
      bartTaskMenu.style.left = `${pos[0] + 30}px`;
      bartTaskMenu.style.top = `${pos[1] - 80}px`;
      bartTaskMenu.classList.remove('hidden');

      if (taskBartTimer) clearInterval(taskBartTimer);
      taskBartTimer = setInterval(() => {
        const freshPos = game.get_bart_screen_pos();
        bartTaskMenu.style.left = `${freshPos[0] + 30}px`;
        bartTaskMenu.style.top = `${freshPos[1] - 80}px`;
      }, 30);
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
