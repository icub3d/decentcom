export function Logo({ size = 32, className = "" }: { size?: number | string; className?: string }) {
  return (
    <svg
      width={size}
      height={size}
      viewBox="0 0 100 100"
      className={`text-ctp-blue ${className}`}
      fill="none"
      xmlns="http://www.w3.org/2000/svg"
    >
      <rect width="100" height="100" rx="22" fill="currentColor" />
      <circle cx="50" cy="38" r="22" stroke="var(--ctp-crust)" strokeWidth="8" />
      <circle cx="50" cy="38" r="8" fill="var(--ctp-crust)" />
      <path
        d="M50 16 L50 24 M28 38 L36 38 M64 38 L72 38"
        stroke="var(--ctp-crust)"
        strokeWidth="8"
        strokeLinecap="round"
      />
      <path
        d="M50 52 L50 82"
        stroke="var(--ctp-crust)"
        strokeWidth="8"
        strokeLinecap="round"
      />
      <path
        d="M50 70 L62 70 M50 82 L62 82"
        stroke="var(--ctp-crust)"
        strokeWidth="8"
        strokeLinecap="round"
      />
    </svg>
  );
}
