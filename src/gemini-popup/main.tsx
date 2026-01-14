import React, { useEffect, useState, useRef } from "react";
import { createRoot } from "react-dom/client";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import ReactDOM from "react-dom/client";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import remarkMath from "remark-math";
import rehypeKatex from "rehype-katex";
import { commands } from "../bindings";
import { formatKeyCombination } from "../lib/utils/keyboard";
import { type } from "@tauri-apps/plugin-os";
import "katex/dist/katex.min.css";

const GeminiPopup: React.FC = () => {
  const [responses, setResponses] = useState<string[]>([]);
  const [loading, setLoading] = useState<boolean>(true);
  const [error, setError] = useState<string | null>(null);
  const [hotkey, setHotkey] = useState<string>("");
  const responseRefs = useRef<(HTMLDivElement | null)[]>([]);
  const lastResponseRef = useRef<string>("");

  useEffect(() => {
    console.log("GeminiPopup component mounted, setting up listeners...");
    
    // Fetch hotkey setting
    const fetchHotkey = async () => {
      try {
        const osType = await type();
        const normalizedOsType = osType === "macos" ? "macos" : osType === "windows" ? "windows" : osType === "linux" ? "linux" : "unknown";
        
        const settingsResult = await commands.getAppSettings();
        if (settingsResult.status === "ok") {
          const bindings = settingsResult.data.bindings || {};
          const transcribeBinding = bindings.transcribe;
          if (transcribeBinding?.current_binding) {
            const formatted = formatKeyCombination(transcribeBinding.current_binding, normalizedOsType);
            setHotkey(formatted);
          }
        }
      } catch (err) {
        console.error("Failed to fetch hotkey:", err);
      }
    };
    
    fetchHotkey();
    
    // Set up handler for direct eval calls - always set it up fresh
    const handleDirectResponse = (responseText: string) => {
      console.log("Received direct response via eval, length:", responseText?.length || 0, "text:", responseText?.substring(0, 50) || "empty");
      console.log("Full response:", responseText);
      if (responseText && typeof responseText === 'string' && responseText.length > 0) {
        // Deduplicate: check if this response was already added
        setResponses(prev => {
          // Check if this exact response already exists in the array
          if (prev.includes(responseText)) {
            console.log("Skipping duplicate response (already in array)");
            return prev;
          }
          // Also check against the last response ref to catch rapid duplicates
          if (responseText === lastResponseRef.current) {
            console.log("Skipping duplicate response (same as last)");
            return prev;
          }
          lastResponseRef.current = responseText;
          console.log("Appending response to list, length:", responseText.length);
          return [...prev, responseText];
        });
        setLoading(false);
        setError(null);
        console.log("State updated - response appended, loading false");
      } else {
        console.warn("Invalid response text received:", responseText, "type:", typeof responseText);
      }
    };
    
    // Register global handler - always update it
    (window as any).__geminiResponseHandler = handleDirectResponse;
    console.log("Registered __geminiResponseHandler");
    
    // Check for pending response
    if ((window as any).__pendingGeminiResponse) {
      console.log("Found pending response, applying it");
      handleDirectResponse((window as any).__pendingGeminiResponse);
      delete (window as any).__pendingGeminiResponse;
    }
    
    // Also listen for custom DOM event
    const handleCustomEvent = (event: CustomEvent) => {
      console.log("Received custom gemini-response event, detail:", event.detail);
      if (event.detail && typeof event.detail === 'string' && event.detail.length > 0) {
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
        // Don't clear responses on error, just show the error
      });

      console.log("Event listeners set up successfully");

      return () => {
        console.log("Cleaning up event listeners");
        unlistenResponse();
        unlistenError();
        window.removeEventListener("gemini-response", handleCustomEvent as EventListener);
        // Don't delete the handler on cleanup - it might be needed for subsequent responses
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

  // Auto-scroll to new response when it's added
  useEffect(() => {
    if (responses.length > 0) {
      const newIndex = responses.length - 1;
      // Use a small delay to ensure the DOM has updated
      const timeoutId = setTimeout(() => {
        const responseElement = responseRefs.current[newIndex];
        if (responseElement) {
          responseElement.scrollIntoView({ behavior: "smooth", block: "start" });
        }
      }, 150);
      
      return () => clearTimeout(timeoutId);
    }
  }, [responses.length]);

  // Debug: log render state
  if (responses.length > 0) {
    console.log("Render - loading:", loading, "responses count:", responses.length, "error:", error);
  }
  
  return (
    <div className="container">
      <div className="header">
        <div className="title">Gemini Response</div>
        <button className="close-btn" id="closeBtn">×</button>
      </div>
      <div className="response-content">
        {loading && responses.length === 0 && (
          <div className="loading">
            {hotkey ? (
              <>Press and hold <strong>{hotkey}</strong> to record your question...</>
            ) : (
              "Waiting for response..."
            )}
          </div>
        )}
        {error && <div className="error">Error: {error}</div>}
        {responses.length > 0 && (
          <div className="markdown-content" style={{ display: "block", visibility: "visible", opacity: 1 }}>
            {responses.map((response, index) => (
              <div 
                key={index} 
                ref={(el) => {
                  responseRefs.current[index] = el;
                }}
                style={{ marginBottom: index < responses.length - 1 ? "24px" : "0" }}
              >
                {index > 0 && <hr style={{ margin: "16px 0", border: "none", borderTop: "1px solid #e0e0e0" }} />}
                <ReactMarkdown
                  remarkPlugins={[remarkGfm, remarkMath]}
                  rehypePlugins={[rehypeKatex]}
                  components={{
                    code({ node, inline, className, children, ...props }: any) {
                      if (inline) {
                        return (
                          <code className="inline-code" {...props}>
                            {children}
                          </code>
                        );
                      }
                      // For code blocks (not inline)
                      return (
                        <pre className="code-block">
                          <code className={className || ''} {...props}>
                            {String(children).replace(/\n$/, '')}
                          </code>
                        </pre>
                      );
                    },
                    pre({ children, ...props }: any) {
                      // If children is a code element, don't wrap it again
                      if (children && typeof children === 'object' && 'type' in children && children.type === 'code') {
                        return <>{children}</>;
                      }
                      return <pre className="code-block" {...props}>{children}</pre>;
                    },
                  }}
                >
                  {response}
                </ReactMarkdown>
              </div>
            ))}
          </div>
        )}
        {!loading && responses.length === 0 && !error && (
          <div className="loading">No response received</div>
        )}
      </div>
    </div>
  );
};

// Prevent multiple root creation - use a module-level variable
let root: ReturnType<typeof ReactDOM.createRoot> | null = null;

const rootElement = document.getElementById("root");
if (rootElement) {
  // Check if root already exists, if not create it
  if (!root) {
    root = ReactDOM.createRoot(rootElement);
  }
  // Render the component (this is safe to call multiple times)
  root.render(<GeminiPopup />);
}
