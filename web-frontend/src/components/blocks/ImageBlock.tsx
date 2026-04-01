import React from 'react';

interface ImageBlockProps {
  url: string;
  alt?: string;
}

const ImageBlock: React.FC<ImageBlockProps> = ({ url, alt }) => {
  return (
    <div className="my-2">
      <img
        src={url}
        alt={alt || 'Image'}
        className="max-w-full h-auto rounded-lg border border-bg-tertiary"
        loading="lazy"
      />
      {alt && <p className="text-xs text-text-muted mt-1">{alt}</p>}
    </div>
  );
};

export default ImageBlock;
