import ErrorComponent from "@/components/error";

export default function Custom404() {
  return (
    <div className="w-full h-full justify-center items-center flex">
      <div>
        <p className="text-center text-3xl">404 - Page Not Found</p>
        <ErrorComponent error={"The page you're looking for does not exist."} />
      </div>

    </div>
  );
}
