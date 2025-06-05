// Inspired by bigskysoftware/fixi
(() => {
    const init = (baseEl) => {
        baseEl.querySelectorAll("[fx-action]").forEach((el) => {
            const url = el.getAttribute("fx-action");
            const method = (el.getAttribute("fx-method") || "GET").toUpperCase();
            const trigger = el.getAttribute("fx-trigger") || "click";
            const targetAttr = el.getAttribute("fx-target");

            const form = el.querySelector("form");

            const main = document.querySelector("main");

            const swap = async (evt) => {
                evt.preventDefault();

                const target = !targetAttr
                    ? el
                    : targetAttr === "none"
                    ? null
                    : document.querySelector(targetAttr);

                const options = {
                    method,
                    headers: { "fx-request": "true" },
                };
                let reqUrl = url;

                if (form) {
                    const params = new URLSearchParams(new FormData(form, evt.submitter));
                    if (method == "GET") {
                        reqUrl += "?" + params;
                    } else {
                        options.body = params;
                    }
                }

                try {
                    const resp = await fetch(reqUrl, options);
                    const text = await resp.text();

                    if (!resp.ok) throw new Error(`${resp.statusText}: ${text}`);

                    if (target) {
                        target.outerHTML = text;
                        main.dispatchEvent(new Event("nomini:process", { bubbles: true }));
                        main.dispatchEvent(new Event("fixi:process", { bubbles: true }));
                    }
                } catch (err) {
                    main.textContent = `Error: ${err.message}`;
                }
            };

            el.addEventListener(trigger, swap);

            setTimeout(() => el.dispatchEvent(new Event("delay")), 2000);
        });
    };

    document.addEventListener("DOMContentLoaded", () => init(document));
    document.addEventListener("fixi:process", (evt) => init(evt.target));
})();
