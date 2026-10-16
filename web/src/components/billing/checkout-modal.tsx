import React, { useState } from 'react';
import {
  X,
  ShieldCheck,
  CreditCard,
  Check,
  Sparkles,
  Lock,
  Zap,
  Info,
  DollarSign,
  AlertCircle,
  Clock,
  Layers,
  FileText,
} from 'lucide-react';

export interface CheckoutModalProps {
  isOpen: boolean;
  onClose: () => void;
  defaultTier?: 'starter' | 'growth' | 'enterprise';
}

export const CheckoutModal: React.FC<CheckoutModalProps> = ({
  isOpen,
  onClose,
  defaultTier = 'growth',
}) => {
  const [tier, setTier] = useState<'starter' | 'growth' | 'enterprise'>(defaultTier);
  const [billingCycle, setBillingCycle] = useState<'monthly' | 'annual'>('annual');
  const [companyName, setCompanyName] = useState('Acme Inc.');
  const [billingEmail, setBillingEmail] = useState('billing@acme.corp');
  const [vatId, setVatId] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [isSuccess, setIsSuccess] = useState(false);

  if (!isOpen) return null;

  // Pricing definitions from PRD §1.5
  const plans = {
    starter: {
      name: 'Starter Tier',
      badge: 'INDIE & AUTOMATORS',
      monthlyPrice: 29,
      annualPrice: 24, // 20% discount ($288/year)
      includedMinutes: 300,
      overageRate: '$0.08 / min',
      resolution: '1080p Full HD (60 fps)',
      features: [
        '300 render minutes / month included',
        'Word-level forced alignment captions',
        'Built-in EBU R128 (-14 LUFS) audio mix',
        'Pre-flight dry-run cost quotes (:validate)',
        'Standard queue processing lane',
        'Community Discord & Email support',
      ],
    },
    growth: {
      name: 'Growth Tier',
      badge: 'MOST POPULAR · TEAMS',
      monthlyPrice: 149,
      annualPrice: 119, // 20% discount ($1,428/year)
      includedMinutes: 3000,
      overageRate: '$0.05 / min',
      resolution: '4K Ultra HD & 1080p (60 fps)',
      features: [
        '3,000 render minutes / month included',
        'Multi-format fan-out (9:16 + 1:1 + 16:9 in 1 job)',
        'Batch campaign CSV runner (up to 10k rows)',
        'Golden frame SSIM visual regression CI',
        'Bring-Your-Own S3 / R2 Bucket storage',
        'Priority worker lanes & 99.5% uptime SLA',
      ],
    },
    enterprise: {
      name: 'Enterprise Tier',
      badge: 'CUSTOM SCALE',
      monthlyPrice: 499,
      annualPrice: 399,
      includedMinutes: 12000,
      overageRate: '$0.03 / min',
      resolution: '4K ProRes / H.264 / AV1 Master',
      features: [
        '12,000+ render minutes included',
        'Dedicated isolated GPU worker clusters',
        'Custom C2PA Content Credentials private key',
        'Regional EU / US data residency pinning',
        '99.9% Uptime SLA & 24/7 PagerDuty support',
        'Custom contract & consolidated invoicing',
      ],
    },
  };

  const selectedPlan = plans[tier];
  const unitPrice = billingCycle === 'annual' ? selectedPlan.annualPrice : selectedPlan.monthlyPrice;
  const subtotal = billingCycle === 'annual' ? unitPrice * 12 : unitPrice;
  const estimatedTax = vatId.trim() ? 0 : Math.round(subtotal * 0.0825 * 100) / 100;
  const totalDueToday = Math.round((subtotal + estimatedTax) * 100) / 100;

  const handleCompleteCheckout = (e: React.FormEvent) => {
    e.preventDefault();
    setIsSubmitting(true);
    setTimeout(() => {
      setIsSubmitting(false);
      setIsSuccess(true);
      setTimeout(() => {
        setIsSuccess(false);
        onClose();
      }, 2000);
    }, 1200);
  };

  return (
    <div
      className="fixed inset-0 z-50 bg-black/80 backdrop-blur-xs flex items-center justify-center p-4 overflow-y-auto select-none"
      onClick={onClose}
    >
      <div
        className="w-full max-w-4xl bg-panel-dark border border-white/12 rounded-xl shadow-2xl overflow-hidden my-8 animate-in fade-in zoom-in-95 duration-150"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Top Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-white/8 bg-canvas-dark">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 rounded-lg bg-amber-safelight/15 border border-amber-safelight/30 flex items-center justify-center">
              <CreditCard className="w-4 h-4 text-amber-safelight" />
            </div>
            <div>
              <h2 className="text-sm font-bold font-mono text-neutral-100 flex items-center gap-2">
                <span>Cutline Engine · Upgrade Plan Checkout</span>
                <span className="px-2 py-0.5 rounded text-[10px] font-mono bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
                  Stripe Test Mode Enabled
                </span>
              </h2>
              <p className="text-xs font-mono text-neutral-500">
                Transparent billing · Zero charge on retried worker failures · Prorated anytime
              </p>
            </div>
          </div>
          <button
            onClick={onClose}
            className="p-1.5 rounded-lg text-neutral-400 hover:text-neutral-200 hover:bg-white/6 transition-colors"
          >
            <X className="w-4 h-4" />
          </button>
        </div>

        {isSuccess ? (
          <div className="p-12 flex flex-col items-center justify-center space-y-4 text-center">
            <div className="w-16 h-16 rounded-full bg-emerald-500/20 text-emerald-400 flex items-center justify-center border border-emerald-500/30">
              <Check className="w-8 h-8" />
            </div>
            <h3 className="text-xl font-bold font-mono text-neutral-100">Subscription Activated!</h3>
            <p className="text-xs font-mono text-neutral-400 max-w-md">
              Your tenant <code className="text-amber-safelight">tenant_acme</code> is now upgraded to the{' '}
              <strong>{selectedPlan.name}</strong>. Your API keys are now active with higher concurrency limits.
            </p>
          </div>
        ) : (
          <div className="grid grid-cols-1 md:grid-cols-12 divide-y md:divide-y-0 md:divide-x divide-white/8">
            {/* Left Column: Plan Configuration & Summary (7 cols) */}
            <div className="md:col-span-7 p-6 space-y-6">
              {/* Billing Cycle Switcher */}
              <div className="flex items-center justify-between p-1 bg-canvas-dark rounded-lg border border-white/6">
                <button
                  type="button"
                  onClick={() => setBillingCycle('annual')}
                  className={`flex-1 py-1.5 px-3 rounded-md font-mono text-xs font-semibold flex items-center justify-center gap-2 transition-all ${
                    billingCycle === 'annual'
                      ? 'bg-amber-safelight text-neutral-950 shadow-sm'
                      : 'text-neutral-400 hover:text-neutral-200'
                  }`}
                >
                  <span>Annual Billing</span>
                  <span className="px-1.5 py-0.2 rounded text-[10px] bg-black/20 text-neutral-950 font-bold">
                    SAVE 20%
                  </span>
                </button>
                <button
                  type="button"
                  onClick={() => setBillingCycle('monthly')}
                  className={`flex-1 py-1.5 px-3 rounded-md font-mono text-xs font-semibold transition-all ${
                    billingCycle === 'monthly'
                      ? 'bg-amber-safelight text-neutral-950 shadow-sm'
                      : 'text-neutral-400 hover:text-neutral-200'
                  }`}
                >
                  Monthly Billing
                </button>
              </div>

              {/* Tier Selection Radio Cards */}
              <div className="space-y-2.5">
                <label className="text-[11px] font-mono text-neutral-400 uppercase tracking-wider">
                  Select Production Tier
                </label>

                {(['starter', 'growth', 'enterprise'] as const).map((tKey) => {
                  const p = plans[tKey];
                  const isSelected = tier === tKey;
                  const price = billingCycle === 'annual' ? p.annualPrice : p.monthlyPrice;

                  return (
                    <div
                      key={tKey}
                      onClick={() => setTier(tKey)}
                      className={`p-3.5 rounded-lg border cursor-pointer transition-all flex items-start justify-between ${
                        isSelected
                          ? 'bg-amber-safelight/10 border-amber-safelight/50 shadow-sm'
                          : 'bg-white/2 border-white/6 hover:bg-white/4 hover:border-white/12'
                      }`}
                    >
                      <div className="space-y-1">
                        <div className="flex items-center gap-2">
                          <span className="font-mono font-bold text-xs text-neutral-100">{p.name}</span>
                          <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-white/6 text-neutral-400 border border-white/6">
                            {p.badge}
                          </span>
                        </div>
                        <p className="text-[11px] font-mono text-neutral-400">
                          {p.includedMinutes.toLocaleString()} min included · {p.resolution}
                        </p>
                      </div>

                      <div className="text-right">
                        <div className="font-mono font-bold text-sm text-neutral-100">
                          ${price}
                          <span className="text-[10px] font-normal text-neutral-400">/mo</span>
                        </div>
                        <div className="text-[10px] font-mono text-neutral-500">
                          {p.overageRate} overage
                        </div>
                      </div>
                    </div>
                  );
                })}
              </div>

              {/* Transparent Add-On Pricing Ledger */}
              <div className="p-3.5 bg-canvas-dark rounded-lg border border-white/6 space-y-2">
                <div className="flex items-center justify-between text-xs font-mono font-semibold text-neutral-300">
                  <span className="flex items-center gap-1.5">
                    <Info className="w-3.5 h-3.5 text-amber-safelight" />
                    <span>Transparent Add-On Rate Card</span>
                  </span>
                  <span className="text-[10px] text-neutral-500">Pay only for what you render</span>
                </div>

                <div className="grid grid-cols-2 gap-2 text-[11px] font-mono text-neutral-400 pt-1 border-t border-white/6">
                  <div>
                    • Multi-Format Fan-Out:{' '}
                    <span className="text-neutral-200 font-bold">$0.02 / extra min</span>
                  </div>
                  <div>
                    • Word-Level Captions:{' '}
                    <span className="text-neutral-200 font-bold">$0.01 / audio-min</span>
                  </div>
                  <div>
                    • C2PA Provenance Signing:{' '}
                    <span className="text-neutral-200 font-bold">$0.005 / video</span>
                  </div>
                  <div>
                    • Pre-Flight Dry Run (:validate):{' '}
                    <span className="text-emerald-400 font-bold">100% Free</span>
                  </div>
                </div>
              </div>

              {/* Zero-Risk Guarantees */}
              <div className="space-y-1.5 text-xs font-mono">
                <div className="flex items-center gap-2 text-emerald-400">
                  <ShieldCheck className="w-4 h-4 shrink-0" />
                  <span>Zero-Charge Policy: Failed or retried worker jobs bill $0.00.</span>
                </div>
                <div className="flex items-center gap-2 text-neutral-400">
                  <Check className="w-4 h-4 shrink-0 text-amber-safelight" />
                  <span>Pre-flight price verification before every job via /v1/renders:validate.</span>
                </div>
                <div className="flex items-center gap-2 text-neutral-400">
                  <Clock className="w-4 h-4 shrink-0 text-amber-safelight" />
                  <span>Cancel anytime with 1-click self-serve prorated credit.</span>
                </div>
              </div>
            </div>

            {/* Right Column: Customer Info, Payment & Summary (5 cols) */}
            <div className="md:col-span-5 p-6 bg-canvas-dark/60 flex flex-col justify-between space-y-6">
              <form onSubmit={handleCompleteCheckout} className="space-y-4">
                <h3 className="text-xs font-mono font-semibold text-neutral-200 uppercase tracking-wider flex items-center gap-1.5">
                  <Lock className="w-3.5 h-3.5 text-amber-safelight" />
                  <span>Secure Checkout (Stripe)</span>
                </h3>

                {/* Company Name */}
                <div className="space-y-1">
                  <label className="text-[11px] font-mono text-neutral-400">Company / Legal Name</label>
                  <input
                    type="text"
                    required
                    value={companyName}
                    onChange={(e) => setCompanyName(e.target.value)}
                    className="w-full px-3 py-1.5 rounded bg-panel-dark border border-white/10 text-neutral-100 font-mono text-xs focus:outline-none focus:border-amber-safelight"
                    placeholder="Acme Corp."
                  />
                </div>

                {/* Billing Email */}
                <div className="space-y-1">
                  <label className="text-[11px] font-mono text-neutral-400">Billing Email Address</label>
                  <input
                    type="email"
                    required
                    value={billingEmail}
                    onChange={(e) => setBillingEmail(e.target.value)}
                    className="w-full px-3 py-1.5 rounded bg-panel-dark border border-white/10 text-neutral-100 font-mono text-xs focus:outline-none focus:border-amber-safelight"
                    placeholder="billing@company.com"
                  />
                </div>

                {/* VAT / Tax ID */}
                <div className="space-y-1">
                  <div className="flex items-center justify-between">
                    <label className="text-[11px] font-mono text-neutral-400">VAT / Tax ID (Optional)</label>
                    <span className="text-[10px] font-mono text-neutral-500">Reverse charge</span>
                  </div>
                  <input
                    type="text"
                    value={vatId}
                    onChange={(e) => setVatId(e.target.value)}
                    className="w-full px-3 py-1.5 rounded bg-panel-dark border border-white/10 text-neutral-100 font-mono text-xs focus:outline-none focus:border-amber-safelight"
                    placeholder="EU123456789 or EIN"
                  />
                </div>

                {/* Simulated Stripe Card Element */}
                <div className="space-y-1">
                  <label className="text-[11px] font-mono text-neutral-400">Payment Method</label>
                  <div className="p-2.5 rounded bg-panel-dark border border-white/10 flex items-center justify-between text-xs font-mono text-neutral-300">
                    <div className="flex items-center gap-2">
                      <CreditCard className="w-4 h-4 text-amber-safelight" />
                      <span>•••• •••• •••• 4242</span>
                    </div>
                    <span className="text-[10px] text-neutral-500">12/28 · CVC 123</span>
                  </div>
                </div>

                {/* Order Summary & Line Items */}
                <div className="pt-3 border-t border-white/8 space-y-1.5 text-xs font-mono">
                  <div className="flex items-center justify-between text-neutral-400">
                    <span>
                      {selectedPlan.name} ({billingCycle === 'annual' ? '12 Months' : '1 Month'})
                    </span>
                    <span className="text-neutral-200">${subtotal.toFixed(2)}</span>
                  </div>

                  {billingCycle === 'annual' && (
                    <div className="flex items-center justify-between text-emerald-400 text-[11px]">
                      <span>Annual Discount (20% Off)</span>
                      <span>-${(selectedPlan.monthlyPrice * 12 - subtotal).toFixed(2)}</span>
                    </div>
                  )}

                  <div className="flex items-center justify-between text-neutral-400 text-[11px]">
                    <span>Estimated Sales Tax / VAT</span>
                    <span>{estimatedTax === 0 ? '$0.00 (exempt)' : `$${estimatedTax.toFixed(2)}`}</span>
                  </div>

                  <div className="flex items-center justify-between text-sm font-bold pt-2 border-t border-white/8 text-neutral-100">
                    <span>Total Due Today</span>
                    <span className="text-amber-safelight">${totalDueToday.toFixed(2)}</span>
                  </div>
                </div>

                {/* Submit Checkout Button */}
                <button
                  type="submit"
                  disabled={isSubmitting}
                  className="w-full py-2.5 px-4 rounded-lg bg-amber-safelight hover:bg-amber-400 text-neutral-950 font-mono font-bold text-xs flex items-center justify-center gap-2 shadow-lg transition-all disabled:opacity-50"
                >
                  {isSubmitting ? (
                    <>
                      <span className="w-3.5 h-3.5 border-2 border-neutral-950 border-t-transparent rounded-full animate-spin" />
                      <span>Activating Subscription...</span>
                    </>
                  ) : (
                    <>
                      <Zap className="w-3.5 h-3.5 fill-current" />
                      <span>Confirm & Activate {selectedPlan.name}</span>
                    </>
                  )}
                </button>
              </form>

              {/* Compliance Badges */}
              <div className="pt-2 border-t border-white/6 flex items-center justify-between text-[10px] font-mono text-neutral-500">
                <span className="flex items-center gap-1">
                  <ShieldCheck className="w-3 h-3 text-emerald-400" />
                  PCI-DSS Level 1
                </span>
                <span>256-bit TLS Encrypted</span>
                <span>Powered by Stripe</span>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
