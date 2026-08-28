import { useState, useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

interface TrackState {
  player_running: boolean;
  title: string;
  artist: string;
  duration: number;
  position: number;
  status: string;
  synced_lyrics: string | null;
  plain_lyrics: string | null;
  lyrics_fetched: boolean;
}

interface LyricLine {
  time: number;
  text: string;
  translation?: string;
}

function parseLrc(lrcText: string): LyricLine[] {
  const lines = lrcText.split("\n");
  const tempMap = new Map<number, string[]>();
  const timeRegex = /\[(\d+):(\d+(?:\.\d+)?)]/;
  
  for (const line of lines) {
    const match = timeRegex.exec(line);
    if (match) {
      const minutes = parseInt(match[1], 10);
      const seconds = parseFloat(match[2]);
      const time = Math.round((minutes * 60 + seconds) * 1000) / 1000;
      const text = line.replace(timeRegex, "").trim();
      
      if (text && !text.startsWith("[") && !text.endsWith("]")) {
        if (!tempMap.has(time)) {
          tempMap.set(time, []);
        }
        tempMap.get(time)!.push(text);
      }
    }
  }
  
  const lyrics: LyricLine[] = [];
  for (const [time, texts] of tempMap.entries()) {
    let mainText = texts[0];
    let transText: string | undefined = undefined;
    
    if (texts.length === 1) {
      const separators = [" / ", " | ", " \\ "];
      for (const sep of separators) {
        if (mainText.includes(sep)) {
          const parts = mainText.split(sep);
          mainText = parts[0].trim();
          transText = parts[1].trim();
          break;
        }
      }
    } else {
      transText = texts.slice(1).join(" / ");
    }
    
    lyrics.push({
      time,
      text: mainText,
      translation: transText,
    });
  }
  
  return lyrics.sort((a, b) => a.time - b.time);
}

function RikaLogo() {
  return (
    <img 
      src="/rika-logo.png" 
      alt="Rika Logo" 
      className="rika-logo-img"
    />
  );
}

export default function App() {
  const [clickThrough, setClickThrough] = useState(false);
  const [state, setState] = useState<TrackState>({
    player_running: false,
    title: "",
    artist: "",
    duration: 0,
    position: 0,
    status: "Stopped",
    synced_lyrics: null,
    plain_lyrics: null,
    lyrics_fetched: false,
  });
  
  const [receivedAt, setReceivedAt] = useState<number>(0);
  const [interpolatedPosition, setInterpolatedPosition] = useState(0);
  const [offsetY, setOffsetY] = useState(0);

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;
    let unlistenShortcut: (() => void) | undefined;

    listen<TrackState>("track-state", (event) => {
      if (!active) return;
      console.log("[Rika Frontend] Received track-state:", event.payload);
      setState(event.payload);
      setReceivedAt(Date.now());
    }).then((fn) => {
      if (!active) {
        fn();
      } else {
        unlisten = fn;
      }
    });

    listen("toggle-click-through", () => {
      if (!active) return;
      console.log("[Rika Frontend] Received toggle-click-through event!");
      toggleClickThrough();
    }).then((fn) => {
      if (!active) {
        fn();
      } else {
        unlistenShortcut = fn;
      }
    });

    return () => {
      active = false;
      if (unlisten) unlisten();
      if (unlistenShortcut) unlistenShortcut();
    };
  }, []);

  const lyrics = useMemo(() => {
    if (state.synced_lyrics) {
      return parseLrc(state.synced_lyrics);
    }
    return [];
  }, [state.synced_lyrics]);

  useEffect(() => {
    if (!state.player_running || state.status !== "Playing") {
      setInterpolatedPosition(state.position);
      return;
    }

    let animationFrameId: number;
    const updatePosition = () => {
      const elapsed = (Date.now() - receivedAt) / 1000;
      const currentPos = Math.min(state.position + elapsed, state.duration);
      setInterpolatedPosition(currentPos);
      animationFrameId = requestAnimationFrame(updatePosition);
    };

    animationFrameId = requestAnimationFrame(updatePosition);
    return () => cancelAnimationFrame(animationFrameId);
  }, [state.position, state.status, receivedAt, state.duration, state.player_running]);

  const handleStartDrag = () => {
    if (!clickThrough) {
      invoke("start_drag");
    }
  };

  const toggleClickThrough = () => {
    console.log("[Rika Frontend] Toggling click through state...");
    setClickThrough((prev) => {
      const nextState = !prev;
      console.log("[Rika Frontend] Invoking set_click_through to:", nextState);
      invoke("set_click_through", { ignore: nextState }).catch(console.error);
      return nextState;
    });
  };

  const handleClose = () => invoke("close_app").catch(console.error);

  const activeIndex = useMemo(() => {
    let index = -1;
    for (let i = 0; i < lyrics.length; i++) {
      if (interpolatedPosition >= lyrics[i].time) {
        index = i;
      } else {
        break;
      }
    }
    return index;
  }, [lyrics, interpolatedPosition]);

  // Adjust scroll offset to keep active lyric centered smoothly
  useEffect(() => {
    if (lyrics.length === 0) return;

    const calculateOffset = () => {
      const targetIndex = activeIndex === -1 ? 0 : activeIndex;
      const activeEl = document.querySelector(`[data-index="${targetIndex}"]`);
      const containerEl = document.querySelector(".rika-lyrics-scroll-container");
      
      if (activeEl && containerEl) {
        const activeTop = (activeEl as HTMLElement).offsetTop;
        const activeHeight = (activeEl as HTMLElement).offsetHeight;
        const containerHeight = containerEl.clientHeight;
        
        // Align active item's center with scroll container's center
        const targetOffset = containerHeight / 2 - activeTop - activeHeight / 2;
        setOffsetY(targetOffset);
      }
    };

    // Calculate immediately and also after a short render delay to ensure sizes are correct
    calculateOffset();
    const timeoutId = setTimeout(calculateOffset, 50);

    window.addEventListener("resize", calculateOffset);
    return () => {
      clearTimeout(timeoutId);
      window.removeEventListener("resize", calculateOffset);
    };
  }, [activeIndex, lyrics]);

  const displayStatus = useMemo(() => {
    if (!state.player_running) {
      return {
        main: "Spotify is not running",
        sub: "Open Spotify to start tracking",
      };
    }

    if (state.status === "Stopped" || !state.title) {
      return {
        main: "No track playing",
        sub: "Play a song on Spotify",
      };
    }

    if (!state.synced_lyrics && !state.plain_lyrics) {
      if (!state.lyrics_fetched) {
        return {
          main: "Searching for lyrics...",
          sub: `${state.title} — ${state.artist}`,
        };
      } else {
        return {
          main: "Lyrics not found",
          sub: `${state.title} — ${state.artist}`,
        };
      }
    }

    return {
      main: "Lyrics not found",
      sub: `${state.title} — ${state.artist}`,
    };
  }, [state]);

  return (
    <div className={`rika-container ${clickThrough ? "locked" : "unlocked"}`}>
      {/* Top Bar (Visible only when unlocked) */}
      {!clickThrough && (
        <div onMouseDown={handleStartDrag} className="rika-header">
          {/* Logo only */}
          <div className="rika-logo-container">
            <RikaLogo />
          </div>

          {/* Window Controls */}
          <div className="rika-controls">
            {/* Window control buttons */}
            <div className="rika-window-btns">
              {/* Subtle lock/click-through button */}
              <button 
                onClick={(e) => { e.stopPropagation(); toggleClickThrough(); }}
                className="rika-window-btn"
                title="Lock HUD (Ctrl+Alt+K)"
              >
                <svg style={{ width: "12px", height: "12px" }} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <rect x="3" y="11" width="18" height="11" rx="2" ry="2" />
                  <path d="M7 11V7a5 5 0 0 1 10 0v4" />
                </svg>
              </button>



              {/* Close */}
              <button 
                onClick={(e) => { e.stopPropagation(); handleClose(); }}
                className="rika-window-btn close"
                title="Close"
              >
                <svg style={{ width: "12px", height: "12px" }} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
                  <line x1="18" y1="6" x2="6" y2="18" />
                  <line x1="6" y1="6" x2="18" y2="18" />
                </svg>
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Main Content Area */}
      <div className="rika-main">
        {lyrics.length === 0 ? (
          <div className="rika-status-container">
            <p className="rika-status-main animate-fade-in-up">
              {displayStatus.main}
            </p>
            {displayStatus.sub && (
              <p className="rika-status-sub animate-fade-in-up">
                {displayStatus.sub}
              </p>
            )}
          </div>
        ) : (
          <div className="rika-lyrics-scroll-container">
            <div 
              className="rika-lyrics-list-inner"
              style={{ transform: `translateY(${offsetY}px)` }}
            >
              {lyrics.map((line, index) => {
                const isActive = index === activeIndex;
                const isPrev = index === activeIndex - 1;
                const isNext = index === activeIndex + 1;
                
                let itemClass = "rika-lyric-item";
                if (isActive) itemClass += " active";
                else if (isPrev) itemClass += " prev";
                else if (isNext) itemClass += " next";
                else if (index < activeIndex) itemClass += " far-prev";
                else itemClass += " far-next";

                return (
                  <div 
                    key={index}
                    className={itemClass}
                    data-index={index}
                  >
                    <p className="rika-lyric-text">
                      {line.text}
                    </p>
                    {line.translation && (
                      <p className="rika-lyric-translation">
                        {line.translation}
                      </p>
                    )}
                  </div>
                );
              })}
            </div>
          </div>
        )}
      </div>

      {/* Floating unlock instruction for clickThrough */}
      {clickThrough && (
        <div className="rika-lock-badge">
          <span className="rika-lock-badge-content">
            Locked
          </span>
        </div>
      )}
    </div>
  );
}