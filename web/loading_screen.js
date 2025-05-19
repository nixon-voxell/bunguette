// Hide loading screen when the game starts.
const observer = new MutationObserver(() => {
    const loading_screen = document.getElementById("loading-screen");
    const bevy = document.getElementById("bevy");
    if (bevy.height > 1) {
        loading_screen.style.display = "none";
        observer.disconnect();
    }
});
observer.observe(bevy, { attributeFilter: ['height'] });
