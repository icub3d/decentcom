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
      <circle cx="50" cy="50" r="28" stroke="var(--ctp-crust)" strokeWidth="8" />
      <circle cx="50" cy="50" r="10" fill="var(--ctp-crust)" />
      <path
        d="M50 18 L50 34 M50 66 L50 82 M18 50 L34 50 M66 50 L82 50"
        stroke="var(--ctp-crust)"
        strokeWidth="8"
        strokeLinecap="round"
      />
    </svg>
  );
}
