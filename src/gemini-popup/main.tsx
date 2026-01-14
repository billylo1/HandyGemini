import React, { useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import ReactDOM from "react-dom/client";

const GeminiPopup: React.FC = () => {
  const [response, setResponse] = useState<string>("");
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    console.log("GeminiPopup component mounted, setting up listeners...");
    
    // Set up handler for direct eval calls
    const handleDirectResponse = (responseText: string) => {
      console.log("Received direct response via eval, length:", responseText.length);
      setResponse(responseText);
      setLoading(false);
      setError(null);
    };
    
    // Register global handler
    (window as any).__geminiResponseHandler = handleDirectResponse;
    
    // Check for pending response
    if ((window as any).__pendingGeminiResponse) {
      console.log("Found pending response, applying it");
      handleDirectResponse((window as any).__pendingGeminiResponse);
      delete (window as any).__pendingGeminiResponse;
    }
    
    // Also listen for custom DOM event
    const handleCustomEvent = (event: CustomEvent) => {
      console.log("Received custom gemini-response event, length:", event.detail?.length || 0);
      if (event.detail) {
        handleDirectResponse(event.detail);
      }
    };
    window.addEventListener("gemini-response", handleCustomEvent as EventListener);
    
    const setupListeners = async () => {
      console.log("Setting up event listeners...");
      
      // Listen for response - use window-specific listener
      const unlistenResponse = await listen<string>("show-response", (event) => {
        console.log("Received show-response event, payload length:", event.payload?.length || 0);
        if (event.payload && event.payload.length > 0) {
          handleDirectResponse(event.payload);
        }
      });

      // Listen for errors
      const unlistenError = await listen<string>("gemini-error", (event) => {
        console.log("Received gemini-error event:", event.payload);
        setError(event.payload);
        setLoading(false);
        setResponse("");
      });

      console.log("Event listeners set up successfully");

      return () => {
        console.log("Cleaning up event listeners");
        unlistenResponse();
        unlistenError();
        window.removeEventListener("gemini-response", handleCustomEvent as EventListener);
        delete (window as any).__geminiResponseHandler;
      };
    };

    const listenersPromise = setupListeners();
    
    listenersPromise.catch((err) => {
      console.error("Failed to setup listeners:", err);
    });
    
    // Close button handler - set up after component mounts
    const handleClose = async () => {
      const appWindow = getCurrentWindow();
      await appWindow.hide();
    };

    // Set up close button after a short delay to ensure DOM is ready
    const closeBtnTimeout = setTimeout(() => {
      const closeBtn = document.getElementById("closeBtn");
      if (closeBtn) {
        closeBtn.addEventListener("click", handleClose);
      }
    }, 100);

    return () => {
      listenersPromise.then((cleanup) => {
        if (cleanup) cleanup();
      });
      clearTimeout(closeBtnTimeout);
      const closeBtn = document.getElementById("closeBtn");
      if (closeBtn) {
        closeBtn.removeEventListener("click", handleClose);
      }
    };
  }, []);

  return (
    <div className="container">
      <div className="header">
        <div className="title">Gemini Response</div>
        <button className="close-btn" id="closeBtn">×</button>
      </div>
      <div className="response-content">
        {loading && !response && <div className="loading">Waiting for response...</div>}
        {error && <div className="error">Error: {error}</div>}
        {response && <div style={{ whiteSpace: "pre-wrap", wordWrap: "break-word" }}>{response}</div>}
      </div>
    </div>
  );
};

const rootElement = document.getElementById("root");
if (rootElement) {
  const root = ReactDOM.createRoot(rootElement);
  root.render(
    <React.StrictMode>
      <GeminiPopup />
    </React.StrictMode>
  );
}
