import {
  AbsoluteFill,
  Easing,
  Img,
  interpolate,
  spring,
  staticFile,
  useCurrentFrame,
  useVideoConfig,
} from "remotion";

const clamp = { extrapolateLeft: "clamp", extrapolateRight: "clamp" } as const;
const ease = Easing.bezier(0.16, 1, 0.3, 1);
const fade = (f: number, a: number, b: number) =>
  interpolate(f, [a, b], [0, 1], { ...clamp, easing: ease });

const MenuPanel: React.FC<{
  children: React.ReactNode;
  x: number;
  y: number;
  w: number;
  from: number;
  delay?: number;
}> = ({ children, x, y, w, from, delay = 0 }) => {
  const frame = useCurrentFrame();
  const p = spring({ frame: frame - from - delay, fps: 30, config: { damping: 150 } });
  const out = interpolate(frame, [210, 235], [1, 0], clamp);

  return (
    <div
      style={{
        position: "absolute",
        left: x,
        top: y,
        width: w,
        opacity: p * out,
        transform: `translateY(${interpolate(p, [0, 1], [-10, 0])}px) scale(${interpolate(p, [0, 1], [0.98, 1])})`,
        transformOrigin: "top center",
        borderRadius: 26,
        padding: "12px 0",
        background: "linear-gradient(145deg, rgba(42,39,46,.9), rgba(13,12,16,.84))",
        border: "1px solid rgba(255,255,255,.16)",
        boxShadow: "0 32px 100px rgba(0,0,0,.52), inset 0 1px 0 rgba(255,255,255,.12)",
        backdropFilter: "blur(28px)",
        overflow: "hidden",
      }}
    >
      {children}
    </div>
  );
};

const Row: React.FC<{
  label: string;
  active?: boolean;
  delay: number;
  chevron?: boolean;
}> = ({ label, active, delay, chevron = true }) => {
  const frame = useCurrentFrame();
  const p = fade(frame, delay, delay + 10);
  return (
    <div
      style={{
        height: 58,
        margin: "4px 12px",
        padding: "0 20px",
        borderRadius: 16,
        display: "flex",
        alignItems: "center",
        justifyContent: "space-between",
        color: "white",
        background: active ? "linear-gradient(90deg,#1778ff,#67b5ff)" : "transparent",
        opacity: p,
        transform: `translateX(${interpolate(p, [0, 1], [-12, 0])}px)`,
        fontSize: 25,
        lineHeight: 1,
        fontWeight: 760,
        letterSpacing: -0.6,
        whiteSpace: "nowrap",
      }}
    >
      <span>{label}</span>
      {chevron ? <span style={{ opacity: 0.82, fontSize: 30 }}>›</span> : null}
    </div>
  );
};

const Terminal: React.FC = () => {
  const frame = useCurrentFrame();
  const p = spring({ frame: frame - 178, fps: 30, config: { damping: 150 } });
  const finalFade = interpolate(frame, [306, 334], [0, 1], clamp);
  const lines = [
    "ssh root@kube-node1",
    "Connecting to kube-node1…",
    "Authenticating with public key…",
    "Welcome to Ubuntu 24.04 LTS",
    "root@kube-node1:~#",
  ];

  return (
    <div
      style={{
        position: "absolute",
        left: 470,
        top: 270,
        width: 980,
        height: 545,
        opacity: p * (1 - finalFade * 0.35),
        transform: `translateY(${interpolate(p, [0, 1], [34, 0])}px) scale(${interpolate(p, [0, 1], [0.97, 1])})`,
        borderRadius: 24,
        overflow: "hidden",
        background: "rgba(11,11,14,.96)",
        border: "1px solid rgba(255,255,255,.14)",
        boxShadow: "0 45px 140px rgba(0,0,0,.62)",
      }}
    >
      <div
        style={{
          height: 54,
          display: "flex",
          alignItems: "center",
          gap: 10,
          paddingLeft: 22,
          background: "linear-gradient(#303038,#24242b)",
          borderBottom: "1px solid rgba(255,255,255,.08)",
        }}
      >
        {["#ff5f57", "#ffbd2e", "#28c840"].map((c) => (
          <div key={c} style={{ width: 14, height: 14, borderRadius: 99, background: c }} />
        ))}
      </div>
      <div
        style={{
          padding: "34px 38px",
          fontFamily: "Menlo, Monaco, Consolas, ui-monospace, monospace",
          color: "#d7f8df",
          fontSize: 30,
          lineHeight: 1.65,
        }}
      >
        {lines.map((line, idx) => (
          <div
            key={line}
            style={{
              opacity: fade(frame, 194 + idx * 18, 202 + idx * 18),
              color: idx === 0 ? "#8ec5ff" : idx === 4 ? "#8dffad" : "#d7f8df",
            }}
          >
            {idx === 0 ? "$ " : ""}
            {line}
            {idx === 4 && frame % 28 < 14 ? <span style={{ color: "white" }}> ▌</span> : null}
          </div>
        ))}
      </div>
    </div>
  );
};

export const ShuttleProductVideo: React.FC = () => {
  const frame = useCurrentFrame();
  const { fps } = useVideoConfig();
  const logo = spring({ frame, fps, config: { damping: 130 } });
  const final = fade(frame, 306, 340);

  return (
    <AbsoluteFill
      style={{
        overflow: "hidden",
        background: "#06070b",
        fontFamily:
          '-apple-system, BlinkMacSystemFont, "SF Pro Display", Inter, Arial, sans-serif',
      }}
    >
      <div
        style={{
          position: "absolute",
          inset: -160,
          background:
            "radial-gradient(circle at 20% 82%, #d091b4 0, transparent 34%), radial-gradient(circle at 76% 22%, #216bff 0, transparent 32%), radial-gradient(circle at 48% 48%, #2c1d37 0, transparent 42%), #06070b",
        }}
      />

      <div
        style={{
          position: "absolute",
          left: 0,
          right: 0,
          top: 0,
          height: 74,
          background: "rgba(8,8,12,.68)",
          borderBottom: "1px solid rgba(255,255,255,.08)",
          backdropFilter: "blur(22px)",
          color: "white",
        }}
      >
        <div
          style={{
            position: "absolute",
            left: "50%",
            top: "50%",
            opacity: logo,
            transform: `translate(-50%, -50%) scale(${interpolate(logo, [0, 1], [0.65, 1])})`,
            display: "flex",
            alignItems: "center",
            gap: 10,
            padding: "8px 16px",
            borderRadius: 22,
            background: frame > 54 && frame < 174 ? "rgba(255,255,255,.16)" : "transparent",
          }}
        >
          <Img src={staticFile("shuttle-icon.png")} style={{ width: 34, height: 34 }} />
          <span style={{ fontSize: 22, fontWeight: 900 }}>Shuttle</span>
        </div>
        <div style={{ position: "absolute", right: 34, top: 24, color: "rgba(255,255,255,.7)", fontSize: 22, fontWeight: 650 }}>
          Fri May 29&nbsp;&nbsp;12:24
        </div>
      </div>

      <MenuPanel x={690} y={86} w={540} from={62}>
        <Row label="Home Lab" delay={68} />
        <Row label="Kubernetes" active={frame >= 116} delay={76} />
        <Row label="Uberspace" delay={84} />
        <Row label="VServer" delay={92} />
        <div style={{ height: 1, margin: "10px 32px", background: "rgba(255,255,255,.13)" }} />
        <Row label="Configuration" delay={100} />
        <Row label="Quit" delay={108} chevron={false} />
      </MenuPanel>

      <MenuPanel x={292} y={218} w={415} from={122}>
        <Row label="Nodes" delay={128} />
        <Row label="kube-node1" active={frame >= 160} delay={136} chevron={false} />
        <Row label="kube-node2" delay={144} chevron={false} />
        <Row label="kube-node3" delay={152} chevron={false} />
      </MenuPanel>

      <Terminal />

      <AbsoluteFill
        style={{
          opacity: final,
          background: `rgba(6,7,11,${interpolate(final, [0, 1], [0, 0.86])})`,
          alignItems: "center",
          justifyContent: "center",
          color: "white",
          textAlign: "center",
        }}
      >
        <div style={{ transform: `scale(${interpolate(final, [0, 1], [0.9, 1])})`, display: "flex", flexDirection: "column", alignItems: "center" }}>
          <Img src={staticFile("shuttle-icon.png")} style={{ width: 148, height: 148, marginBottom: 28 }} />
          <div style={{ fontSize: 104, fontWeight: 980, letterSpacing: -5 }}>Shuttle</div>
          <div style={{ marginTop: 16, fontSize: 38, color: "rgba(255,255,255,.72)", fontWeight: 500 }}>
            SSH, commands, and URLs from your menu bar.
          </div>
        </div>
      </AbsoluteFill>
    </AbsoluteFill>
  );
};
