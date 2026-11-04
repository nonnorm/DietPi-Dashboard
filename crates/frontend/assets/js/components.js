async function getUpdateMessage(oldVersion) {
    const now = Math.round(Date.now() / 1000);
    let { newVersion = null, lastChecked = 0 } = JSON.parse(
        localStorage.getItem("update-check") || "{}"
    );

    if (now - lastChecked > 86400) {
        const resp = await fetch("https://api.github.com/repos/ravenclaw900/DietPi-Dashboard/tags");
        const json = await resp.json();

        // Remove preceding 'v'
        newVersion = json[0].name.substring(1);

        localStorage.setItem("update-check", JSON.stringify({ newVersion, lastChecked: now }));
    }

    if (
        newVersion.localeCompare(oldVersion, undefined, {
            numeric: true,
        }) === 1
    ) {
        return window.__dashboardI18n?.template("new_version_available", {
            version: newVersion,
        }) ?? `New version available: ${newVersion}`;
    }
}

(() => {
    customElements.define(
        "web-terminal",
        class extends HTMLElement {
            async connectedCallback() {
                const term = new Terminal();

                const fitAddon = new FitAddon.FitAddon();
                term.loadAddon(fitAddon);

                term.open(this);

                fetch("/terminal/stream").then(async (res) => {
                    for await (const chunk of res.body) {
                        term.write(chunk)
                    }
                });

                term.onResize((dimensions) => {
                    fetch("/terminal/resize", { method: "POST", body: new URLSearchParams(dimensions) });
                });

                fitAddon.fit()
                window.addEventListener("resize", () => fitAddon.fit(), 50);

                let sendTimeout = null;
                let sendBuf = "";

                // I would MUCH rather stream this, but that's not possible on HTTP/1.1 and doesn't work at all on Firefox
                term.onData((data) => {
                    clearTimeout(sendTimeout);
                    sendBuf += data;
                    // Short debounce to prevent excess requests
                    sendTimeout = setTimeout(() => {
                        fetch("/terminal/write", { method: "POST", body: sendBuf });
                        sendBuf = "";
                    }, 60);
                });
            }
        }
    );

    customElements.define(
        "code-editor",
        class extends HTMLElement {
            connectedCallback() {
                const textarea = this.querySelector("textarea");
                const pre = this.querySelector("pre");

                const highlight = () => {
                    pre.textContent = textarea.value;
                    microlight(pre);
                };

                const autosize = () => {
                    textarea.style.height = "0px";
                    textarea.style.height = textarea.scrollHeight + "px";
                    pre.style.height = textarea.scrollHeight + "px";
                };

                textarea.addEventListener("input", highlight);
                textarea.addEventListener("input", autosize);

                highlight();
                autosize();
            }
        }
    );
})();

(() => {
    const I18N_MESSAGES = {
        en: {
            app_name: "DietPi Dashboard",
            backend: "Backend",
            toggle_navigation: "Toggle navigation",
            messages: "Messages",
            toggle_language: "Toggle language",
            toggle_theme: "Toggle theme",
            close_navigation: "Close navigation",
            login_form_title: "Login Form",
            password_placeholder: "Password",
            login: "Login",
            nav_system: "System",
            nav_processes: "Processes",
            nav_software: "Software",
            nav_services: "Services",
            nav_management: "Management",
            nav_terminal: "Terminal",
            nav_file_browser: "File Browser",
            footer_product: "DietPi Dashboard",
            footer_by: "by",
            footer_design_by: "WebUI Design by",
            footer_repo_title: "DietPi-Dashboard Repository",
            dietpi_update_available: "DietPi Update Available: {version}",
            system_overview: "System Overview",
            cpu_load: "CPU Load",
            memory_label: "Memory",
            swap_label: "Swap",
            peak_disk: "Peak Disk",
            pressure_monitor: "Pressure monitor",
            cpu_statistics: "CPU Statistics",
            memory_usage: "Memory Usage",
            disk_usage: "Disk Usage",
            cpu_graph: "CPU Graph",
            temperature_graph: "Temperature Graph",
            memory_graph: "Memory Graph",
            network_graph: "Network Graph",
            terminal_title: "Terminal",
            cpu: "CPU",
            temperature: "Temperature",
            ram: "RAM",
            swap: "Swap",
            sent: "Sent",
            received: "Received",
            services_title: "Services",
            error_log: "Error Log",
            start_time: "Start Time",
            view_log: "View log",
            active: "Active",
            inactive: "Inactive",
            failed: "Failed",
            unknown: "Unknown",
            software_title: "Software",
            description: "Description",
            dependencies: "Dependencies",
            docs: "Docs",
            install_software_tab: "Install Software",
            uninstall_software_tab: "Uninstall Software",
            install: "Install",
            uninstall: "Uninstall",
            operation_summary: "Operation Summary",
            file_name: "File Name",
            file_size: "File Size",
            save: "Save",
            rename: "Rename",
            delete: "Delete",
            download: "Download",
            refresh: "Refresh",
            hide_hidden_files: "Hide Hidden Files",
            show_hidden_files: "Show Hidden Files",
            new_file: "New File",
            new_folder: "New Folder",
            upload: "Upload",
            enter_new_name: "Enter a new name:",
            confirm_delete_file: "Are you sure you want to delete this file?",
            confirm_delete_folder: "Are you sure you want to delete this folder?",
            enter_file_name: "Enter a file name:",
            enter_folder_name: "Enter a folder name:",
            host_information: "Host Information",
            hostname: "Hostname",
            network_interface: "Network Interface",
            uptime: "Uptime",
            installed_packages: "Installed Packages",
            os_version: "OS Version",
            kernel_version: "Kernel Version",
            dietpi_version: "DietPi Version",
            architecture: "Architecture",
            frontend_config: "Frontend Config",
            backend_config: "Backend Config",
            dashboard_administration: "Dashboard Administration",
            logout: "Logout",
            processes_title: "Processes",
            rows: "Rows",
            rows_comfortable: "Comfortable",
            rows_compact: "Compact",
            per_page: "Per Page",
            pid: "PID",
            name: "Name",
            status: "Status",
            cpu_usage: "CPU Usage",
            ram_usage: "RAM Usage",
            actions: "Actions",
            no_process_data: "No process data available",
            kill_process: "Kill process",
            terminate_process: "Terminate process",
            resume_process: "Resume process",
            pause_process: "Pause process",
            first: "First",
            prev: "Prev",
            next: "Next",
            last: "Last",
            running: "Running",
            paused: "Paused",
            sleeping: "Sleeping",
            other: "Other",
        },
        zh: {
            app_name: "DietPi 仪表盘",
            backend: "后端",
            toggle_navigation: "切换导航",
            messages: "消息",
            toggle_language: "切换语言",
            toggle_theme: "切换主题",
            close_navigation: "关闭导航",
            login_form_title: "登录",
            password_placeholder: "密码",
            login: "登录",
            nav_system: "系统",
            nav_processes: "进程",
            nav_software: "软件",
            nav_services: "服务",
            nav_management: "管理",
            nav_terminal: "终端",
            nav_file_browser: "文件浏览器",
            footer_product: "DietPi 仪表盘",
            footer_by: "作者",
            footer_design_by: "界面设计",
            footer_repo_title: "DietPi-Dashboard 仓库",
            dietpi_update_available: "可用 DietPi 更新：{version}",
            system_overview: "系统总览",
            cpu_load: "CPU 负载",
            memory_label: "内存",
            swap_label: "交换分区",
            peak_disk: "峰值磁盘",
            pressure_monitor: "压力监测",
            cpu_statistics: "CPU 统计",
            memory_usage: "内存使用",
            disk_usage: "磁盘使用",
            cpu_graph: "CPU 图表",
            temperature_graph: "温度图表",
            memory_graph: "内存图表",
            network_graph: "网络图表",
            terminal_title: "终端",
            cpu: "CPU",
            temperature: "温度",
            ram: "内存",
            swap: "交换分区",
            sent: "发送",
            received: "接收",
            services_title: "服务",
            error_log: "错误日志",
            start_time: "启动时间",
            view_log: "查看日志",
            active: "运行中",
            inactive: "未运行",
            failed: "失败",
            unknown: "未知",
            software_title: "软件",
            description: "描述",
            dependencies: "依赖",
            docs: "文档",
            install_software_tab: "安装软件",
            uninstall_software_tab: "卸载软件",
            install: "安装",
            uninstall: "卸载",
            operation_summary: "操作摘要",
            file_name: "文件名",
            file_size: "文件大小",
            save: "保存",
            rename: "重命名",
            delete: "删除",
            download: "下载",
            refresh: "刷新",
            hide_hidden_files: "隐藏隐藏文件",
            show_hidden_files: "显示隐藏文件",
            new_file: "新建文件",
            new_folder: "新建文件夹",
            upload: "上传",
            enter_new_name: "输入新名称：",
            confirm_delete_file: "确定删除该文件吗？",
            confirm_delete_folder: "确定删除该文件夹吗？",
            enter_file_name: "输入文件名：",
            enter_folder_name: "输入文件夹名：",
            host_information: "主机信息",
            hostname: "主机名",
            network_interface: "网络接口",
            uptime: "运行时长",
            installed_packages: "已安装软件包",
            os_version: "系统版本",
            kernel_version: "内核版本",
            dietpi_version: "DietPi 版本",
            architecture: "架构",
            frontend_config: "前端配置",
            backend_config: "后端配置",
            dashboard_administration: "控制台管理",
            logout: "退出登录",
            processes_title: "进程",
            rows: "行高",
            rows_comfortable: "舒适",
            rows_compact: "紧凑",
            per_page: "每页",
            pid: "进程号",
            name: "名称",
            status: "状态",
            cpu_usage: "CPU 占用",
            ram_usage: "内存占用",
            actions: "操作",
            no_process_data: "暂无进程数据",
            kill_process: "强制结束进程",
            terminate_process: "终止进程",
            resume_process: "恢复进程",
            pause_process: "暂停进程",
            first: "首页",
            prev: "上一页",
            next: "下一页",
            last: "末页",
            running: "运行中",
            paused: "已暂停",
            sleeping: "休眠",
            other: "其他",
        },
    };

    const I18N_TEMPLATES = {
        en: {
            process_summary: ({ start = 0, end = 0, total = 0 }) =>
                `Showing ${start}-${end} of ${total} processes`,
            page_of: ({ page = 1, totalPages = 1 }) => `Page ${page} / ${totalPages}`,
            temperature_value: ({ value = "--" }) => `Temperature: ${value}`,
            dietpi_update_available: ({ version = "" }) => `DietPi Update Available: ${version}`,
            cpu_temperature_value: ({ value = "--" }) => `CPU Temperature: ${value}`,
            global_cpu_value: ({ value = "--" }) => `Global CPU: ${value}`,
            cpu_core_usage: ({ core = "", value = "--" }) => `CPU ${core}: ${value}`,
            ram_usage_value: ({ value = "--" }) => `RAM Usage: ${value}`,
            swap_usage_value: ({ value = "--" }) => `Swap Usage: ${value}`,
            disk_usage_value: ({ name = "", mount = "", value = "--" }) =>
                `${name} (${mount}): ${value}`,
            new_version_available: ({ version = "" }) => `New version available: ${version}`,
        },
        zh: {
            process_summary: ({ start = 0, end = 0, total = 0 }) =>
                `显示第 ${start}-${end} 条，共 ${total} 个进程`,
            page_of: ({ page = 1, totalPages = 1 }) => `第 ${page} / ${totalPages} 页`,
            temperature_value: ({ value = "--" }) => `温度：${value}`,
            dietpi_update_available: ({ version = "" }) => `可用 DietPi 更新：${version}`,
            cpu_temperature_value: ({ value = "--" }) => `CPU 温度：${value}`,
            global_cpu_value: ({ value = "--" }) => `总 CPU：${value}`,
            cpu_core_usage: ({ core = "", value = "--" }) => `CPU ${core}：${value}`,
            ram_usage_value: ({ value = "--" }) => `内存使用：${value}`,
            swap_usage_value: ({ value = "--" }) => `交换分区使用：${value}`,
            disk_usage_value: ({ name = "", mount = "", value = "--" }) =>
                `${name}（${mount}）：${value}`,
            new_version_available: ({ version = "" }) => `发现新版本：${version}`,
        },
    };

    const normalizeLang = (lang) =>
        String(lang || "").toLowerCase().startsWith("zh") ? "zh" : "en";

    const detectLang = () => {
        try {
            return normalizeLang(localStorage.getItem("dashboardLang") || navigator.language);
        } catch (_) {
            return "en";
        }
    };

    const t = (lang, key, fallback = "") =>
        I18N_MESSAGES[lang]?.[key] ?? I18N_MESSAGES.en[key] ?? fallback;

    const template = (lang, key, data = {}) => {
        const fn = I18N_TEMPLATES[lang]?.[key] ?? I18N_TEMPLATES.en[key];
        if (typeof fn !== "function") return "";
        return fn(data);
    };

    const collect = (root, selector) => {
        const nodes = [];
        if (root?.nodeType === 1 && root.matches(selector)) {
            nodes.push(root);
        }
        if (root?.querySelectorAll) {
            nodes.push(...root.querySelectorAll(selector));
        }
        return nodes;
    };

    let isApplying = false;

    const applyI18n = (lang, root = document) => {
        if (!root || isApplying) return;

        isApplying = true;
        try {
            for (const el of collect(root, "[data-i18n]")) {
                const key = el.dataset.i18n;
                if (!key) continue;
                el.textContent = t(lang, key, el.textContent || "");
            }

            for (const el of collect(root, "[data-i18n-title]")) {
                const key = el.dataset.i18nTitle;
                if (!key) continue;
                el.setAttribute("title", t(lang, key, el.getAttribute("title") || ""));
            }

            for (const el of collect(root, "[data-i18n-aria-label]")) {
                const key = el.dataset.i18nAriaLabel;
                if (!key) continue;
                el.setAttribute("aria-label", t(lang, key, el.getAttribute("aria-label") || ""));
            }

            for (const el of collect(root, "[data-i18n-placeholder]")) {
                const key = el.dataset.i18nPlaceholder;
                if (!key) continue;
                el.setAttribute("placeholder", t(lang, key, el.getAttribute("placeholder") || ""));
            }

            for (const el of collect(root, "[data-i18n-template]")) {
                const key = el.dataset.i18nTemplate;
                if (!key) continue;
                const rendered = template(lang, key, el.dataset);
                if (rendered) {
                    el.textContent = rendered;
                }
            }

            for (const el of collect(root, "[data-lang-value]")) {
                el.textContent = lang === "zh" ? "中" : "EN";
            }

            if (root === document || root === document.body) {
                document.title = t(lang, "app_name", document.title);
            }
        } finally {
            isApplying = false;
        }
    };

    const setLang = (lang) => {
        const next = normalizeLang(lang || detectLang());
        const root = document.documentElement;
        root.dataset.lang = next;
        root.lang = next === "zh" ? "zh-CN" : "en";

        try {
            localStorage.setItem("dashboardLang", next);
        } catch (_) {}

        applyI18n(next, document);
        return next;
    };

    window.__dashboardI18n = {
        t: (key, fallback = "") => t(normalizeLang(document.documentElement.dataset.lang), key, fallback),
        template: (key, data) =>
            template(normalizeLang(document.documentElement.dataset.lang), key, data),
    };

    window.__applyDashboardI18n = (lang, root = document) =>
        applyI18n(normalizeLang(lang || document.documentElement.dataset.lang || detectLang()), root);

    window.__setDashboardLang = (lang) => setLang(lang);

    const init = () => {
        setLang(document.documentElement.dataset.lang || detectLang());

        let rafId = 0;
        const observer = new MutationObserver((mutations) => {
            if (rafId || isApplying) return;

            const addedNodes = [];
            for (const mutation of mutations) {
                for (const node of mutation.addedNodes) {
                    if (node.nodeType === 1) {
                        addedNodes.push(node);
                    }
                }
            }

            if (addedNodes.length === 0) return;

            rafId = requestAnimationFrame(() => {
                rafId = 0;
                const lang = normalizeLang(document.documentElement.dataset.lang);
                for (const node of addedNodes) {
                    applyI18n(lang, node);
                }
            });
        });
        observer.observe(document.body, { childList: true, subtree: true });
    };

    if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", init);
    } else {
        init();
    }
})();
