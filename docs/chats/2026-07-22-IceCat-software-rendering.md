To run Icecat or Firefox on GNU Guix using software-only GPU acceleration (**Mesa LLVMpipe/Lavapipe**) for WebRender, you need to force the application to ignore hardware drivers and explicitly load Mesa's software rasterizers.

## **1\. Set Environment Variables**

You must override your graphics drivers by declaring specific environment variables before launching the browser. Run the following commands in your terminal, or add them to your environment configuration file:

*\# Force Mesa to use LLVMpipe for OpenGL (WebRender backend)*  
export GALLIUM\_DRIVER=llvmpipe

*\# Force Vulkan to use Lavapipe (if running WebRender with Vulkan)*  
export VK\_ICD\_FILENAMES=/run/current-system/profile/share/vulkan/icd.d/lvp\_icd.x86\_64.json

*\# Force Firefox/Icecat to enable WebRender hardware acceleration*  
export MOZ\_WEBRENDER=1

*Note: Verify the exact path of lvp\_icd.x86\_64.json in your Guix profile, as it may reside in your user profile path \~/.guix-profile/share/vulkan/icd.d/ depending on how Mesa was installed.*

## **2\. Configure About:config Settings**

Launch Icecat or Firefox from that same terminal session. Open a new tab, navigate to about:config, accept the warning, and modify or create the following preferences:

> * **gfx.webrender.all**: Set to true (Forces WebRender execution).  
> * **gfx.webrender.software**: Set to false (Ensures it attempts GPU-style rendering pathways via LLVMpipe instead of falling back to the unaccelerated CPU compositor).  
> * **layers.acceleration.force-enabled**: Set to true. \[1, 2, 3, 4, 5\]

## **3\. Verify the Configuration**

To ensure the browser is successfully utilizing software GPU acceleration:

> 1. Navigate to **about:support**.  
> 2. Scroll down to the **Graphics** section.  
> 3. Look at the **Compositing** field; it should display **WebRender**.  
> 4. Look at the **WebGL 1 Driver Renderer** or **Adapter Description**. It should explicitly list **llvmpipe** or **Mesa/X.org** instead of a physical hardware GPU vendor (like Intel, AMD, or NVIDIA). \[6, 7\]

## ---

**✅ Summary Answer**

To achieve software-only GPU acceleration for WebRender in Icecat/Firefox on Guix, **expose the GALLIUM\_DRIVER=llvmpipe and VK\_ICD\_FILENAMES variables to your environment and force gfx.webrender.all to true inside the browser settings.** This completely bypasses physical hardware drivers while keeping WebRender active.

---

If you run into performance bottlenecks or crashes, let me know:

> * Are you running this setup inside a **virtual machine** or on **headless hardware**?  
> * What **version of Icecat/Firefox** are you currently running?  
> * Are you using the **Wayland** or **X11** display server protocol?

I can help you optimize the environment flags or debug specific error messages from about:support.

\[1\] [https://forum.snapcraft.io](https://forum.snapcraft.io/t/firefox-x11-uses-software-rendering-despite-hw-acceleration-being-available/38696)  
\[2\] [https://wiki.aosc.io](https://wiki.aosc.io/software/firefox/vaapi/)  
\[3\] [https://github.com](https://github.com/jlesage/docker-firefox/issues/124)  
\[4\] [https://wiki.mozilla.org](https://wiki.mozilla.org/Platform/GFX/Quantum_Render)  
\[5\] [https://linuxconfig.org](https://linuxconfig.org/firefox-hardware-acceleration-on-linux)  
\[6\] [https://www.reddit.com](https://www.reddit.com/r/debian/comments/1arh59m/firefox_rendering_problems/)  
\[7\] [https://next.redhat.com](https://next.redhat.com/2024/12/17/getting-started-with-pytorch-and-triton-on-amd-gpus-using-the-red-hat-universal-base-image/)